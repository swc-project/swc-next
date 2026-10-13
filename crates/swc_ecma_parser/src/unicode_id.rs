pub fn can_start_id(cp: u32) -> bool {
    cp == u32::from(b'$')
        || cp == u32::from(b'_')
        || char::from_u32(cp).is_some_and(unicode_id_start::is_id_start)
}

pub fn can_continue_id(cp: u32) -> bool {
    cp == u32::from(b'$')
        || cp == u32::from(b'_')
        || cp == 0x200c
        || cp == 0x200d
        || char::from_u32(cp).is_some_and(unicode_id_start::is_id_continue)
}
