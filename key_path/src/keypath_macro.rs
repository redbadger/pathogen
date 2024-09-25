#[macro_export]
macro_rules! keypath {
    // Start of path for Vec<T>
    ($type:path : [$index:expr] $($tail:tt)* ) => {
        {
            use $crate::IndexNavigable;

            type TheVec = $type;
            let path = TheVec::index_keypath_segment($index);
            keypath![path $($tail)*]
        }
    };
    // Start of path for T
    ($type:path : $($tail:tt)* ) => {
        {
            type Local = $type;
            let paths = Local::keypaths();
            keypath![paths $($tail)*]
        }
    };
    // Field access
    ($path:ident . $field:ident $($tail:tt)*) => {
        {
            let path = $path.fields().$field;
            keypath![path $($tail)*]
        }
    };
    // Tuple access
    ($path:ident . $field:tt $($tail:tt)*) => {
        {
            let path = $path.$field;
            keypath![path $($tail)*]
        }
    };
    // Index access
    ($path:ident [$index:expr] $($tail:tt)*) => {
        {
            let path = $path.at($index);
            keypath![path $($tail)*]
        }
    };
    // Direct field access
    ($path:ident $field:ident $($tail:tt)*) => {
        {
            let path = $path.$field;
            keypath![path $($tail)*]
        }
    };
    // End of path
    ($path:ident) => { $path };
}
