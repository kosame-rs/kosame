use std::cell::RefCell;

use syn::Ident;

use crate::{clause::WithItem, command::Command, visitor::Visitor};

thread_local! {
    /// Usage of this field is unsafe, see comment below.
    static COMMAND_TREE: RefCell<Option<&'static CommandTree<'static>>> = const { RefCell::new(None) };
    static STACK: RefCell<Option<usize>> = const { RefCell::new(None) };
}

#[derive(Default)]
pub struct CommandTree<'a> {
    nodes: Vec<Node<'a>>,
}

impl<'a> CommandTree<'a> {
    fn find_node(&self, command: &Command) -> &Node<'a> {
        self.nodes
            .iter()
            .find(|node| std::ptr::eq(node.command, command))
            .expect("start node not found")
    }

    fn find_node_index(&self, command: &Command) -> usize {
        self.nodes
            .iter()
            .position(|node| std::ptr::eq(node.command, command))
            .expect("start node not found")
    }

    pub fn scope(&self, f: impl FnOnce()) {
        COMMAND_TREE.with_borrow_mut(|option| {
            if option.is_some() {
                panic!("nested command tree scopes are not allowed");
            }

            *option = Some(unsafe {
                // Safety: The only access point for this command tree is the `with` method called
                // within the stack frame of `f`. The `with` method reverts the static lifetime to
                // a locally scoped one again.
                std::mem::transmute::<&CommandTree<'a>, &'static CommandTree<'static>>(self)
            });
        });
        f();
        COMMAND_TREE.with_borrow_mut(|option| *option = None);
    }

    pub fn with<R>(f: impl FnOnce(&CommandTree<'_>) -> R) -> R {
        COMMAND_TREE.with_borrow(|command_tree| match command_tree {
            Some(command_tree) => f(command_tree),
            None => panic!("called `CommandTree::with` outside command tree scope"),
        })
    }

    pub fn command_scope(&self, command: &Command, f: impl FnOnce()) {
        let index = self.find_node_index(command);
        let previous = STACK.with_borrow_mut(|stack| stack.replace(index));
        f();
        STACK.with_borrow_mut(|stack| *stack = previous);
    }

    pub fn with_command<R>(f: impl FnOnce(&Command) -> R) -> R {
        Self::with(|command_tree| {
            let Some(command) = STACK.with_borrow(|command| command.clone()) else {
                panic!("called `CommandTree::with_command` outside of a command scope")
            };

            let command = &command_tree.nodes[command];
            f(command.command)
        })
    }

    pub fn parent(&self, command: &Command) -> Option<&Command> {
        self.find_node(command)
            .parent
            .map(|index| self.nodes[index].command)
    }

    // pub fn find_with_item(&self, start: &Command, alias: &Ident) -> Option<&'a WithItem> {
    //     let mut current = self.find_node(start);
    //     loop {
    //         if let Some(with) = &current.command.with {
    //             for item in &with.items {
    //                 if &item.alias.name == alias {
    //                     return Some(item);
    //                 }
    //             }
    //         }
    //         let Some(parent) = current.parent else { break };
    //         current = &self.nodes[parent];
    //     }
    //     None
    // }
}

struct Node<'a> {
    parent: Option<usize>,
    command: &'a Command,
}

#[derive(Default)]
pub struct CommandTreeBuilder<'a> {
    tree: CommandTree<'a>,
    stack: Vec<usize>,
}

impl<'a> CommandTreeBuilder<'a> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn build(self) -> CommandTree<'a> {
        self.tree
    }
}

impl<'a> Visitor<'a> for CommandTreeBuilder<'a> {
    fn visit_command(&mut self, command: &'a Command) {
        self.tree.nodes.push(Node::<'a> {
            parent: self.stack.last().copied(),
            command,
        });
        self.stack.push(self.tree.nodes.len() - 1);
    }

    fn end_command(&mut self) {
        self.stack.pop();
    }
}
