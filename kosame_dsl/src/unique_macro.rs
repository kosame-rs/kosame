macro_rules! unique_macro {
    ($format:literal, $span:expr) => {{
        let span = $span;
        static AUTO_INCREMENT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        let increment = AUTO_INCREMENT.fetch_add(1, Ordering::Relaxed);
        let file = span.file();
        let line_column = span.start();
        let hash = {
            let mut hasher = std::hash::DefaultHasher::new();
            file.hash(&mut hasher);
            line_column.line.hash(&mut hasher);
            line_column.column.hash(&mut hasher);
            increment.hash(&mut hasher);
            hasher.finish()
        };
        let unique_macro_name = format_ident!($format, hash);
        unique_macro_name
    }};
}

pub(crate) use unique_macro;
