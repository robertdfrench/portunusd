/// Derive a single `From<Error>` implementation.
///
/// This macro derives a simple `From<Error>` implementation, making it easier to use the `?`
/// operator. If you encounter a `Result` type that you don't want to fool with, you can use this
/// macro to bundle it into a more general enum.
///
/// Most of the time, you should prefer to use the `define_error_enum!` macro to define an entire
/// suite of mappings at once.
///
/// # Example
/// ```
/// use std::fs::File;
/// use std::io::Write;
/// use portunusd::map_error;
///
/// pub enum CrabToFileError {
///     Io(std::io::Error),
///     Utf8(std::string::FromUtf8Error)
/// }
///
/// map_error!(std::io::Error => CrabToFileError as Io);
/// map_error!(std::string::FromUtf8Error => CrabToFileError as Utf8);
///
/// fn write_crab_to_file(mut file: File) -> Result<(),CrabToFileError> {
///     let crab_emoji = String::from_utf8(vec![0xF0, 0x9F, 0xA6, 0x80])?;
///     write!(&mut file, "Crab Emoji: {}", crab_emoji)?;
///     Ok(())
/// }
/// ```
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


/// Derive a suite of `From<Error>` implementations.
///
/// This macro derives a simple Error type with a suite of `From<Error>` implementation, making it
/// easier to use the `?` operator. If you encounter `Result` types that you don't want to fool
/// with, you can use this macro to bundle them into a more general enum.
///
/// # Example
/// ```
/// use std::fs::File;
/// use std::io::Write;
/// use portunusd::define_error_enum;
///
/// define_error_enum!(
///     pub enum CrabToFileError {
///         Io(std::io::Error),
///         Utf8(std::string::FromUtf8Error)
///     }
/// );
///
/// fn write_crab_to_file(mut file: File) -> Result<(),CrabToFileError> {
///     let crab_emoji = String::from_utf8(vec![0xF0, 0x9F, 0xA6, 0x80])?;
///     write!(&mut file, "Crab Emoji: {}", crab_emoji)?;
///     Ok(())
/// }
/// ```
#[macro_export]
macro_rules! define_error_enum {
    (pub enum $dest:ident {$label_0:ident($source_0:ty)$(, $label_n:ident($source_n:ty))*}) => {
        #[derive(Debug)]
        pub enum $dest {
            $label_0($source_0),
            $($label_n($source_n),)*
        }
        portunusd::map_error!($source_0 => $dest as $label_0);
        $(portunusd::map_error!($source_n => $dest as $label_n);)*
    }
}
