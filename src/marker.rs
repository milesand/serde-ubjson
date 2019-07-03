macro_rules! make_consts {
    ( $($name:ident = $val:expr);* $(;)? ) => {
        $(
            pub(crate) const $name: u8 = $val;
        )*
    };
}

make_consts! {
    NULL = b'Z';
    NOOP = b'N';
    TRUE = b'T';
    FALSE = b'F';
    I8 = b'i';
    U8 = b'U';
    I16 = b'I';
    I32 = b'l';
    I64 = b'L';
    F32 = b'd';
    F64 = b'D';
    HI_PRECISION = b'H';
    CHAR = b'C';
    STRING = b'S';
    ARR_START = b'[';
    ARR_END = b']';
    OBJ_START = b'{';
    OBJ_END = b'}';
    TYPE = b'$';
    LENGTH = b'#';
}
