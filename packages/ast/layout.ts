/** Sizes and sentinels shared by the base columns and appended rows. */
export const NONE = 0xffffffff;
export const SPAN_SIZE = 8;
export const PAYLOAD_SIZE = 16;
export const ROW_SIZE = 32;
export const EXTRA_SIZE = 4;
// A 32-byte row cannot use this capacity bit within the 32-bit wire limit.
export const UTF8_STRINGS = 0x80000000;
