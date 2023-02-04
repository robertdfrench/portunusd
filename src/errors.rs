#[macro_export]
macro_rules! map_error {
    ($source:ty => $dest:ty as $label:ident) => {
        impl From<$source> for $dest {
            fn from(source: $source) -> Self {
                <$dest>::$label(source)
            }
        }
    }
}

#[macro_export]
macro_rules! define_error_enum {
    (pub enum $dest:ident {$label_0:ident($source_0:ty)$(, $label_n:ident($source_n:ty))*}) => {
        #[derive(Debug)]
        pub enum $dest {
            $label_0($source_0),
            $($label_n($source_n),)*
        }
        map_error!($source_0 => $dest as $label_0);
        $(map_error!($source_n => $dest as $label_n);)*
    }
}
