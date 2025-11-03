use std::{cell::RefCell, collections::HashMap};

use crate::{
    clause::{FromItem, WithItem},
    command::Command,
    visitor::Visitor,
};

thread_local! {
    /// Usage of this field is unsafe, see comment below.
    static PARENT_MAP: RefCell<Option<&'static ParentMap<'static>>> = const { RefCell::new(None) };
}

#[derive(Default)]
pub struct ParentMap<'a> {
    map: HashMap<*const (), Parent<'a>>,
}

impl<'a> ParentMap<'a> {
    pub fn scope(&self, f: impl FnOnce()) {
        PARENT_MAP.with_borrow_mut(|option| {
            if option.is_some() {
                panic!("nested parent map scopes are not allowed");
            }

            *option = Some(unsafe {
                // Safety: The only access point for this parent map is the `with` method called
                // within the stack frame of `f`. The `with` method reverts the static lifetime to
                // a locally scoped one again.
                std::mem::transmute::<&ParentMap<'a>, &'static ParentMap<'static>>(self)
            });
        });
        f();
        PARENT_MAP.with_borrow_mut(|option| *option = None);
    }

    pub fn with<R>(f: impl FnOnce(&ParentMap<'_>) -> R) -> R {
        PARENT_MAP.with_borrow(|parent_map| match parent_map {
            Some(parent_map) => f(parent_map),
            None => panic!("called `ParentMap::with` outside parent map scope"),
        })
    }

    pub fn parent<N>(&self, node: &'a N) -> Option<&Parent<'a>>
    where
        Parent<'a>: From<&'a N>,
    {
        let ptr = node as *const N as *const ();
        self.map.get(&ptr)
    }

    pub fn seek_parent<N, P>(&self, node: &'a N) -> Option<&'a P>
    where
        P: 'a,
        Parent<'a>: From<&'a N>,
        for<'b> &'a P: TryFrom<&'b Parent<'a>>,
    {
        let mut ptr = node as *const N as *const ();
        while let Some(parent) = self.map.get(&ptr) {
            if let Ok(parent) = <&'a P>::try_from(parent) {
                return Some(parent);
            }
            ptr = parent.as_ptr();
        }
        None
    }
}

#[derive(Clone)]
pub enum Parent<'a> {
    Command(&'a Command),
    FromItem(&'a FromItem),
    WithItem(&'a WithItem),
}

impl Parent<'_> {
    fn as_ptr(&self) -> *const () {
        match self {
            Self::Command(inner) => *inner as *const Command as *const (),
            Self::FromItem(inner) => *inner as *const FromItem as *const (),
            Self::WithItem(inner) => *inner as *const WithItem as *const (),
        }
    }
}

impl<'a> From<&'a Command> for Parent<'a> {
    fn from(v: &'a Command) -> Self {
        Self::Command(v)
    }
}

impl<'a> From<&'a FromItem> for Parent<'a> {
    fn from(v: &'a FromItem) -> Self {
        Self::FromItem(v)
    }
}

impl<'a> From<&'a WithItem> for Parent<'a> {
    fn from(v: &'a WithItem) -> Self {
        Self::WithItem(v)
    }
}

impl<'a> TryFrom<&Parent<'a>> for &'a Command {
    type Error = ();
    fn try_from(value: &Parent<'a>) -> Result<Self, Self::Error> {
        match value {
            Parent::Command(inner) => Ok(inner),
            _ => Err(()),
        }
    }
}

impl<'a> TryFrom<&Parent<'a>> for &'a FromItem {
    type Error = ();
    fn try_from(value: &Parent<'a>) -> Result<Self, Self::Error> {
        match value {
            Parent::FromItem(inner) => Ok(inner),
            _ => Err(()),
        }
    }
}

impl<'a> TryFrom<&Parent<'a>> for &'a WithItem {
    type Error = ();
    fn try_from(value: &Parent<'a>) -> Result<Self, Self::Error> {
        match value {
            Parent::WithItem(inner) => Ok(inner),
            _ => Err(()),
        }
    }
}

#[derive(Default)]
pub struct ParentMapBuilder<'a> {
    parent_map: ParentMap<'a>,
    stack: Vec<Parent<'a>>,
}

impl<'a> ParentMapBuilder<'a> {
    pub fn new() -> Self {
        Default::default()
    }

    fn push<N>(&mut self, node: &'a N)
    where
        &'a N: Into<Parent<'a>>,
    {
        let ptr = node as *const N as *const ();
        if let Some(parent) = self.stack.last()
            && self.parent_map.map.insert(ptr, parent.clone()).is_some()
        {
            panic!("node has multiple parents");
        }
        self.stack.push(node.into());
    }

    pub fn build(self) -> ParentMap<'a> {
        self.parent_map
    }
}

impl<'a> Visitor<'a> for ParentMapBuilder<'a> {
    fn visit_command(&mut self, node: &'a Command) {
        self.push(node);
    }

    fn end_command(&mut self) {
        self.stack.pop();
    }

    fn visit_from_item(&mut self, node: &'a FromItem) {
        self.push(node);
    }

    fn end_from_item(&mut self) {
        self.stack.pop();
    }

    fn visit_with_item(&mut self, node: &'a WithItem) {
        self.push(node);
    }

    fn end_with_item(&mut self) {
        self.stack.pop();
    }
}
