use std::default::Default;
use std::io::{Read, Write};
use std::thread::sleep;
use tracing::log::debug;
use ironrdp_pdu::cursor::{ReadCursor, WriteCursor};
use ironrdp_pdu::{cast_length, ensure_fixed_part_size, ensure_size, PduDecode, PduEncode, PduError, PduResult, utils};
use ironrdp_pdu::utils::CharacterSet;
use ironrdp_svc::SvcPduEncode;
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive as _, ToPrimitive as _};
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct RailOrderType(u16);
macro_rules! rail_order_formats {
    (
        $(
            ($konst:ident, $num:expr);
        )+
    ) => {
        impl RailOrderType {
        $(
            pub const $konst:RailOrderType = RailOrderType($num);
        )+

            fn as_str(&self) -> Option<&'static str> {
                match self.0 {
                    $(
                        $num => Some(stringify!($konst)),
                    )+
                        _ => None
                }
            }
        }
    }
}
rail_order_formats! {
    (UNKOWN, 0x0000);
    (ORDER_EXEC, 0x0001);
    (ORDER_ACTIVATE, 0x0002);
    (ORDER_SYSPARAM, 0x0003);
    (ORDER_SYSCOMMAND, 0x0004);
    (ORDER_HANDSHAKE, 0x0005);
    (ORDER_NOTIFY_EVENT, 0x0006);
    (ORDER_WINDOWMOVE, 0x0008);
    (ORDER_LOCALMOVESIZE, 0x0009);
    (ORDER_MINMAXINFO, 0x000A);
    (ORDER_CLIENTSTATUS, 0x000B);
    (ORDER_SYSMENU, 0x000C);
    (ORDER_LANGBARINFO, 0x000D);
    (ORDER_GET_APPID_REQ, 0x000E);
    (ORDER_GET_APPID_RESP, 0x000F);
    (ORDER_TASKBARINFO, 0x0010);
    (ORDER_LANGUAGEIMEINFO, 0x0011);
    (ORDER_COMPARTMENTINFO, 0x0012);
    (ORDER_HANDSHAKE_EX, 0x0013);
    (ORDER_ZORDER_SYNC, 0x0014);
    (ORDER_CLOAK, 0x0015);
    (ORDER_POWER_DISPLAY_REQUEST, 0x0016);
    (ORDER_SNAP_ARRANGE, 0x0017);
    (ORDER_GET_APPID_RESP_EX, 0x0018);
    (ORDER_EXEC_RESULT, 0x0080);
}


#[derive(Debug)]
pub enum RailDataPdu {
    Unkown,

    EXEC(RailExecOrder),
    ACTIVATE,
    SYSPARAM(RailSysParamOrder),
    SYSCOMMAND,
    HANDSHAKE {
        builder_number: u32
    },
    NOTIFY_EVENT,
    WINDOWMOVE,
    LOCALMOVESIZE,
    MINMAXINFO,
    CLIENTSTATUS {
        flags: u32
    },
    SYSMENU,
    LANGBARINFO,
    GET_APPID_REQ,
    GET_APPID_RESP,
    TASKBARINFO,
    LANGUAGEIMEINFO,
    COMPARTMENTINFO,
    HANDSHAKE_EX { builder_number: u32, rail_handshake_flags: u32 },
    ZORDER_SYNC,
    CLOAK,
    POWER_DISPLAY_REQUEST,
    SNAP_ARRANGE,
    GET_APPID_RESP_EX,
    EXEC_RESULT(RailExecResult),
}
#[derive(Debug)]
pub struct RailExecResult {
    flags: u16,
    result: u16,
    raw: u32,
    reserved: u16,
    file_length: u16,
    file: String,
}
const TS_RAIL_ORDER_EXEC: u16 = 0x0001;
const TS_RAIL_ORDER_ACTIVATE: u16 = 0x0002;
const TS_RAIL_ORDER_SYSPARAM: u16 = 0x0003;
const TS_RAIL_ORDER_SYSCOMMAND: u16 = 0x0004;
const TS_RAIL_ORDER_HANDSHAKE: u16 = 0x0005;
const TS_RAIL_ORDER_NOTIFY_EVENT: u16 = 0x0006;
const TS_RAIL_ORDER_WINDOWMOVE: u16 = 0x0008;
const TS_RAIL_ORDER_LOCALMOVESIZE: u16 = 0x0009;
const TS_RAIL_ORDER_MINMAXINFO: u16 = 0x000A;
const TS_RAIL_ORDER_CLIENTSTATUS: u16 = 0x000B;
const TS_RAIL_ORDER_SYSMENU: u16 = 0x000C;
const TS_RAIL_ORDER_LANGBARINFO: u16 = 0x000D;
const TS_RAIL_ORDER_GET_APPID_REQ: u16 = 0x000E;
const TS_RAIL_ORDER_GET_APPID_RESP: u16 = 0x000F;
const TS_RAIL_ORDER_TASKBARINFO: u16 = 0x0010;
const TS_RAIL_ORDER_LANGUAGEIMEINFO: u16 = 0x0011;
const TS_RAIL_ORDER_COMPARTMENTINFO: u16 = 0x0012;
const TS_RAIL_ORDER_HANDSHAKE_EX: u16 = 0x0013;
const TS_RAIL_ORDER_ZORDER_SYNC: u16 = 0x0014;
const TS_RAIL_ORDER_CLOAK: u16 = 0x0015;
const TS_RAIL_ORDER_POWER_DISPLAY_REQUEST: u16 = 0x0016;
const TS_RAIL_ORDER_SNAP_ARRANGE: u16 = 0x0017;
const TS_RAIL_ORDER_GET_APPID_RESP_EX: u16 = 0x0018;

const TS_RAIL_ORDER_EXEC_RESULT: u16 = 0x0080;
impl RailDataPdu {}
impl PduDecode<'_> for RailDataPdu {
    fn decode(src: &mut ReadCursor<'_>) -> PduResult<Self> {
        // ensure_fixed_part_size!(in: src);
        ensure_size!(in: src, size: 4);
        let header = RailPDUHeader::decode(src);
        match header {
            Ok(header) => {
                match header.OrderType {
                    RailOrderType::ORDER_HANDSHAKE => {
                        let builder_number = src.read_u32();
                        return Ok(RailDataPdu::HANDSHAKE { builder_number });
                    }
                    RailOrderType::ORDER_HANDSHAKE_EX => {
                        let builder_number = src.read_u32();
                        let rail_handshake_flags = src.read_u32();
                        return Ok(RailDataPdu::HANDSHAKE_EX { builder_number, rail_handshake_flags });
                    }
                    RailOrderType::ORDER_SYSPARAM => {
                        debug!("RAILDataPDU ORDER_SYSPARAM");
                        let s = RailSysParamOrder::decode(src)?;
                        return Ok(RailDataPdu::SYSPARAM(s));
                    }
                    RailOrderType::ORDER_EXEC_RESULT => {
                        debug!("RAILDataPDU ORDER_EXEC_RESULT");

                        let mut result = RailExecResult {
                            flags: src.read_u16(),
                            result: src.read_u16(),
                            raw: src.read_u32(),
                            reserved: src.read_u16(),
                            file_length: src.read_u16(),
                            file: "".to_string(),
                        };
                        result.file = utils::read_string_from_cursor(src, CharacterSet::Unicode, true)?;
                        Ok(RailDataPdu::EXEC_RESULT(result))
                    }

                    (ty) => {
                        debug!("Rail RAILDataPDU  未知命令{:?}",ty.0);
                        Ok(RailDataPdu::Unkown)
                    }
                }
            }
            (s) => {
                debug!("Rail RAILDataPDU  无效命令");
                Ok(RailDataPdu::Unkown)
            }
        }
    }
}
impl PduEncode for RailDataPdu {
    fn encode(&self, dst: &mut WriteCursor<'_>) -> PduResult<()> {
        match self {
            RailDataPdu::Unkown => {
                Ok(())
            }
            RailDataPdu::HANDSHAKE { builder_number } => {
                let mut header = RailPDUHeader {
                    OrderType: RailOrderType::ORDER_HANDSHAKE,
                    OrderLength: 4,
                };

                header.encode(dst)?;
                dst.write_u32(*builder_number);
                Ok(())
            }
            RailDataPdu::HANDSHAKE_EX { builder_number, rail_handshake_flags: railHandshakeFlags } => {
                let mut header = RailPDUHeader {
                    OrderType: RailOrderType::ORDER_HANDSHAKE,
                    OrderLength: 4,
                };

                header.encode(dst)?;
                dst.write_u32(*builder_number);
                dst.write_u32(*railHandshakeFlags);
                Ok(())
            }
            RailDataPdu::CLIENTSTATUS { flags } => {
                let mut header = RailPDUHeader {
                    OrderType: RailOrderType::ORDER_CLIENTSTATUS,
                    OrderLength: 4,
                };
                header.encode(dst)?;
                dst.write_u32(*flags);
                Ok(())
            }

            RailDataPdu::SYSPARAM(order) => {
                let length = order.size();
                let mut header = RailPDUHeader {
                    OrderType: RailOrderType::ORDER_SYSPARAM,
                    OrderLength: length as u16,
                };
                header.encode(dst)?;
                order.encode(dst)
            }
            RailDataPdu::EXEC(order) => {
                let length = order.size();
                let mut header = RailPDUHeader {
                    OrderType: RailOrderType::ORDER_SYSPARAM,
                    OrderLength: length as u16,
                };
                header.encode(dst)?;
                order.encode(dst)
            }
            _ => {
                Ok(())
            }
        }
    }

    fn name(&self) -> &'static str {
        return "RaiDataPdu";
    }

    fn size(&self) -> usize {
        match self {
            RailDataPdu::Unkown => {
                0
            }
            RailDataPdu::HANDSHAKE { builder_number } => {
                RailPDUHeader::FIXED_PART_SIZE + 4
            }
            RailDataPdu::HANDSHAKE_EX { .. } => {
                RailPDUHeader::FIXED_PART_SIZE + 8
            }
            RailDataPdu::CLIENTSTATUS { flags } => {
                RailPDUHeader::FIXED_PART_SIZE + 4
            }
            RailDataPdu::SYSPARAM(order) => {
                let length = order.size();
                RailPDUHeader::FIXED_PART_SIZE + length
            }
            RailDataPdu::EXEC(order) => {
                let length = order.size();
                RailPDUHeader::FIXED_PART_SIZE + length
            }
            _ => {
                0
            }
        }
    }
}

impl SvcPduEncode for RailDataPdu {}
#[derive(Debug)]
pub struct Rectangle16 {
    pub left: u16,
    pub top: u16,
    pub right: u16,
    pub bottom: u16,
}
#[derive(Debug)]
pub struct RailHighContrast {
    pub flags: u32,
    pub colorSchemeLength: u32,
    pub colorScheme: String,
}
#[derive(Debug)]
pub struct TsFilterKeys {
    Flags: u32,
    WaitTime: u32,
    DelayTime: u32,
    RepeatTime: u32,
    BounceTime: u32,
}
#[derive(Debug)]
pub struct RailSysParamOrder {
    param: u32,
    pub params: u32,
    dragFullWindows: u8,
    keyboardCues: u8,
    keyboardPref: u8,
    mouseButtonSwap: u8,
    workArea: Rectangle16,
    displayChange: Rectangle16,
    taskbarPos: Rectangle16,
    pub highContrast: RailHighContrast,
    caretWidth: u32,
    stickyKeys: u32,
    toggleKeys: u32,
    filterKeys: TsFilterKeys,
    setScreenSaveActive: u8,
    setScreenSaveSecure: u8,
}

/*Bit mask values for SPI_ parameters*/
const SPI_MASK_SET_DRAG_FULL_WINDOWS: u32 = 0x00000001;
const SPI_MASK_SET_KEYBOARD_CUES: u32 = 0x00000002;
const SPI_MASK_SET_KEYBOARD_PREF: u32 = 0x00000004;
const SPI_MASK_SET_MOUSE_BUTTON_SWAP: u32 = 0x00000008;
const SPI_MASK_SET_WORK_AREA: u32 = 0x00000010;
const SPI_MASK_DISPLAY_CHANGE: u32 = 0x00000020;
const SPI_MASK_TASKBAR_POS: u32 = 0x00000040;
const SPI_MASK_SET_HIGH_CONTRAST: u32 = 0x00000080;
const SPI_MASK_SET_SCREEN_SAVE_ACTIVE: u32 = 0x00000100;
const SPI_MASK_SET_SCREEN_SAVE_SECURE: u32 = 0x00000200;
const SPI_MASK_SET_CARET_WIDTH: u32 = 0x00000400;
const SPI_MASK_SET_STICKY_KEYS: u32 = 0x00000800;
const SPI_MASK_SET_TOGGLE_KEYS: u32 = 0x00001000;
const SPI_MASK_SET_FILTER_KEYS: u32 = 0x00002000;

const SPI_SET_DRAG_FULL_WINDOWS: u32 = 0x00000025;
const SPI_SET_KEYBOARD_CUES: u32 = 0x0000100B;
const SPI_SET_KEYBOARD_PREF: u32 = 0x00000045;
const SPI_SET_MOUSE_BUTTON_SWAP: u32 = 0x00000021;
const SPI_SET_WORK_AREA: u32 = 0x0000002F;
const SPI_DISPLAY_CHANGE: u32 = 0x0000F001;
const SPI_TASKBAR_POS: u32 = 0x0000F000;
const SPI_SET_HIGH_CONTRAST: u32 = 0x00000043;
const SPI_SET_CARET_WIDTH: u32 = 0x00002007;
const SPI_SET_STICKY_KEYS: u32 = 0x0000003B;
const SPI_SET_TOGGLE_KEYS: u32 = 0x00000035;
const SPI_SET_FILTER_KEYS: u32 = 0x00000033;
const SPI_SET_SCREEN_SAVE_ACTIVE: u32 = 0x00000011;
const SPI_SET_SCREEN_SAVE_SECURE: u32 = 0x00000077;
impl Default for RailSysParamOrder {
    fn default() -> Self {
        RailSysParamOrder {
            param: 0,
            params: 0,
            dragFullWindows: 0,
            keyboardCues: 0,
            keyboardPref: 0,
            mouseButtonSwap: 0,
            workArea: Rectangle16 {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            },
            displayChange: Rectangle16 {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            },
            taskbarPos: Rectangle16 {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            },
            highContrast: RailHighContrast {
                flags: 0,
                colorSchemeLength: 0,
                colorScheme: "".to_string(),
            },
            caretWidth: 0,
            stickyKeys: 0,
            toggleKeys: 0,
            filterKeys: TsFilterKeys {
                Flags: 0,
                WaitTime: 0,
                DelayTime: 0,
                RepeatTime: 0,
                BounceTime: 0,
            },
            setScreenSaveActive: 0,
            setScreenSaveSecure: 0,
        }
    }
}

impl RailSysParamOrder {
    pub const FIXED_PART_SIZE: usize = 1;
    pub fn new() -> Self {
        RailSysParamOrder::default()
    }
    pub fn new_with_high_contrast(rail_high_contrast: RailHighContrast) -> Self {
        let mut order = RailSysParamOrder::new();
        order.params != SPI_MASK_SET_HIGH_CONTRAST;
        order.param = SPI_SET_HIGH_CONTRAST;
        order.highContrast = rail_high_contrast;
        order
    }
    pub fn new_with_mouse_button_swap(mouse_button_swap: u8) -> Self {
        let mut order = RailSysParamOrder::new();
        order.params != SPI_MASK_SET_MOUSE_BUTTON_SWAP;
        order.param = SPI_SET_MOUSE_BUTTON_SWAP;
        order.mouseButtonSwap = mouse_button_swap;
        order
    }
    pub fn new_with_keyboard_pref(keyboard_pref: u8) -> Self {
        let mut order = RailSysParamOrder::new();
        order.params != SPI_MASK_SET_KEYBOARD_PREF;
        order.param = SPI_SET_KEYBOARD_PREF;
        order.keyboardPref = keyboard_pref;
        order
    }
    pub fn new_with_drag_full_windows(drag_full_windows: u8) -> Self {
        let mut order = RailSysParamOrder::new();
        order.params != SPI_MASK_SET_DRAG_FULL_WINDOWS;
        order.param = SPI_SET_DRAG_FULL_WINDOWS;
        order.dragFullWindows = drag_full_windows;
        order
    }
    pub fn new_with_keyboard_cues(keyboard_cues: u8) -> Self {
        let mut order = RailSysParamOrder::new();
        order.params != SPI_MASK_SET_KEYBOARD_CUES;
        order.param = SPI_SET_KEYBOARD_CUES;
        order.keyboardCues = keyboard_cues;
        order
    }
    pub fn new_with_work_area(work_area: Rectangle16) -> Self {
        let mut order = RailSysParamOrder::new();
        order.params != SPI_MASK_SET_WORK_AREA;
        order.param = SPI_SET_WORK_AREA;
        order.workArea = work_area;
        order
    }
    pub fn new_with_display_change(displayChange: Rectangle16) -> Self {
        let mut order = RailSysParamOrder::new();
        order.params != SPI_MASK_DISPLAY_CHANGE;
        order.param = SPI_DISPLAY_CHANGE;
        order.displayChange = displayChange;
        order
    }
    pub fn new_with_taskbar_pos(taskbarPos: Rectangle16) -> Self {
        let mut order = RailSysParamOrder::new();
        order.params != SPI_MASK_DISPLAY_CHANGE;
        order.param = SPI_DISPLAY_CHANGE;
        order.taskbarPos = taskbarPos;
        order
    }
    pub fn new_with_filter_keys(filterKeys: TsFilterKeys) -> Self {
        let mut order = RailSysParamOrder::new();
        order.params != SPI_MASK_SET_FILTER_KEYS;
        order.param = SPI_SET_FILTER_KEYS;
        order.filterKeys = filterKeys;
        order
    }
    pub fn new_with_sticky_keys(stickyKeys: u32) -> Self {
        let mut order = RailSysParamOrder::new();
        order.params != SPI_MASK_SET_STICKY_KEYS;
        order.param = SPI_SET_STICKY_KEYS;
        order.stickyKeys = stickyKeys;
        order
    }
    pub fn new_with_caret_width(caretWidth: u32) -> Self {
        let mut order = RailSysParamOrder::new();
        order.params != SPI_MASK_SET_CARET_WIDTH;
        order.param = SPI_SET_CARET_WIDTH;
        order.caretWidth = caretWidth;
        order
    }
    pub fn new_with_toggle_keys(toggleKeys: u32) -> Self {
        let mut order = RailSysParamOrder::new();
        order.params != SPI_MASK_SET_TOGGLE_KEYS;
        order.param = SPI_SET_TOGGLE_KEYS;
        order.toggleKeys = toggleKeys;
        order
    }
    pub fn new_with_scree_save_active(scree_save_active: u8) -> Self {
        let mut order = RailSysParamOrder::new();
        order.params != SPI_MASK_SET_SCREEN_SAVE_ACTIVE;
        order.param = SPI_SET_SCREEN_SAVE_ACTIVE;
        order.setScreenSaveActive = scree_save_active;
        order
    }
    pub fn new_with_scree_save_secure(scree_save_secure: u8) -> Self {
        let mut order = RailSysParamOrder::new();
        order.params != SPI_MASK_SET_SCREEN_SAVE_SECURE;
        order.param = SPI_SET_SCREEN_SAVE_SECURE;
        order.setScreenSaveSecure = scree_save_secure;
        order
    }
}
impl PduEncode for RailSysParamOrder {
    fn encode(&self, dst: &mut WriteCursor<'_>) -> PduResult<()> {
        dst.write_u32(self.param);
        match self.param {
            SPI_SET_DRAG_FULL_WINDOWS => {
                dst.write_u8(self.dragFullWindows)
            }
            SPI_SET_KEYBOARD_CUES => {
                dst.write_u8(self.keyboardCues)
            }
            SPI_SET_KEYBOARD_PREF => {
                dst.write_u8(self.keyboardPref)
            }
            SPI_SET_MOUSE_BUTTON_SWAP => {
                dst.write_u8(self.mouseButtonSwap)
            }
            SPI_SET_WORK_AREA => {
                dst.write_u16(self.workArea.left);
                dst.write_u16(self.workArea.top);
                dst.write_u16(self.workArea.right);
                dst.write_u16(self.workArea.bottom);
            }
            SPI_DISPLAY_CHANGE => {
                dst.write_u16(self.displayChange.left);
                dst.write_u16(self.displayChange.top);
                dst.write_u16(self.displayChange.right);
                dst.write_u16(self.displayChange.bottom);
            }
            SPI_TASKBAR_POS => {
                dst.write_u16(self.taskbarPos.left);
                dst.write_u16(self.taskbarPos.top);
                dst.write_u16(self.taskbarPos.right);
                dst.write_u16(self.taskbarPos.bottom);
            }
            SPI_SET_HIGH_CONTRAST => {
                dst.write_u32(self.highContrast.flags);
                dst.write_u32(self.highContrast.colorSchemeLength);
                let s = &self.highContrast.colorScheme;
                dst.write_slice(s.as_bytes());
            }
            SPI_SETFILTERKEYS => {
                dst.write_u32(self.filterKeys.Flags);
                dst.write_u32(self.filterKeys.WaitTime);
                dst.write_u32(self.filterKeys.DelayTime);
                dst.write_u32(self.filterKeys.RepeatTime);
                dst.write_u32(self.filterKeys.BounceTime);
            }
            SPI_SETSTICKYKEYS => {
                dst.write_u32(self.stickyKeys)
            }
            SPI_SETCARETWIDTH => {
                dst.write_u32(self.caretWidth)
            }
            SPI_SETTOGGLEKEYS => {
                dst.write_u32(self.toggleKeys)
            }
            // SPI_MASK_SET_SET_SCREEN_SAVE_SECURE => {
            //     dst.write_u8(self.setScreenSaveSecure)
            // }
            // SPI_MASK_SET_SCREEN_SAVE_ACTIVE => {
            //     dst.write_u8(self.setScreenSaveActive)
            // }
            _ => {}
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        return "RailSystemParam";
    }

    fn size(&self) -> usize {
        let mut length: usize = 4;
        match self.param {
            SPI_SET_DRAG_FULL_WINDOWS => {
                length += 1
            }
            SPI_SET_KEYBOARD_CUES => {
                length += 1
            }
            SPI_SET_KEYBOARD_PREF => {
                length += 1
            }
            SPI_SET_MOUSE_BUTTON_SWAP => {
                length += 1
            }
            SPI_SET_WORK_AREA => {
                length += 8
            }
            SPI_DISPLAY_CHANGE => {
                length += 8
            }
            SPI_TASKBAR_POS => {
                length += 8
            }
            SPI_SET_HIGH_CONTRAST => {
                length += 8;
                let s = &self.highContrast.colorScheme;
                length += s.as_bytes().len()
            }
            SPI_SET_FILTER_KEYS => {
                length += (4 * 5)
            }
            SPI_SET_STICKY_KEYS => {
                length += 4
            }
            SPI_SET_CARET_WIDTH => {
                length += 4
            }
            SPI_SET_TOGGLE_KEYS => {
                length += 4
            }
            // SPI_MASK_SET_SET_SCREEN_SAVE_SECURE => {
            //     dst.write_u8(self.setScreenSaveSecure)
            // }
            // SPI_MASK_SET_SCREEN_SAVE_ACTIVE => {
            //     dst.write_u8(self.setScreenSaveActive)
            // }
            _ => {}
        }
        length
    }
}
impl PduDecode<'_> for RailSysParamOrder {
    fn decode(src: &mut ReadCursor<'_>) -> PduResult<Self> {
        let param = src.read_u32();
        match param {
            SPI_SET_DRAG_FULL_WINDOWS => {
                Ok(RailSysParamOrder::new_with_drag_full_windows(src.read_u8()))
            }
            SPI_SET_KEYBOARD_CUES => {
                Ok(RailSysParamOrder::new_with_keyboard_cues(src.read_u8()))
            }
            SPI_SET_KEYBOARD_PREF => {
                Ok(RailSysParamOrder::new_with_keyboard_pref(src.read_u8()))
            }
            SPI_SET_MOUSE_BUTTON_SWAP => {
                Ok(RailSysParamOrder::new_with_mouse_button_swap(src.read_u8()))
            }
            SPI_SET_WORK_AREA => {
                Ok(RailSysParamOrder::new_with_work_area(
                    Rectangle16 {
                        left: src.read_u16(),
                        top: src.read_u16(),
                        right: src.read_u16(),
                        bottom: src.read_u16(),
                    }
                ))
            }
            SPI_DISPLAY_CHANGE => {
                Ok(RailSysParamOrder::new_with_display_change(Rectangle16 {
                    left: src.read_u16(),
                    top: src.read_u16(),
                    right: src.read_u16(),
                    bottom: src.read_u16(),
                }))
            }
            SPI_TASKBAR_POS => {
                Ok(RailSysParamOrder::new_with_taskbar_pos(Rectangle16 {
                    left: src.read_u16(),
                    top: src.read_u16(),
                    right: src.read_u16(),
                    bottom: src.read_u16(),
                }))
            }
            SPI_SET_HIGH_CONTRAST => {
                let mut contrast = RailHighContrast {
                    flags: src.read_u32(),
                    colorSchemeLength: src.read_u32(),
                    colorScheme: "".to_string(),
                };
                contrast.colorScheme = utils::read_string_from_cursor(src, CharacterSet::Unicode, true)?;
                Ok(RailSysParamOrder::new_with_high_contrast(
                    contrast
                ))
            }
            SPI_SET_FILTER_KEYS => {
                Ok(RailSysParamOrder::new_with_filter_keys(TsFilterKeys {
                    Flags: src.read_u32(),
                    WaitTime: src.read_u32(),
                    DelayTime: src.read_u32(),
                    RepeatTime: src.read_u32(),
                    BounceTime: src.read_u32(),
                }))
            }
            SPI_SET_STICKY_KEYS => {
                Ok(RailSysParamOrder::new_with_sticky_keys(src.read_u32()))
            }
            SPI_SET_CARET_WIDTH => {
                Ok(RailSysParamOrder::new_with_caret_width(src.read_u32()))
            }
            SPI_SET_TOGGLE_KEYS => {
                Ok(RailSysParamOrder::new_with_toggle_keys(src.read_u32()))
            }
            SPI_SET_SCREEN_SAVE_ACTIVE => {
                Ok(RailSysParamOrder::new_with_scree_save_active(src.read_u8()))
            }
            SPI_SET_SCREEN_SAVE_SECURE => {
                Ok(RailSysParamOrder::new_with_scree_save_secure(src.read_u8()))
            }
            _ => {
                Ok(RailSysParamOrder::new())
            }
        }
    }
}

pub const TS_RAIL_CLIENTSTATUS_ALLOWLOCALMOVESIZE: u32 = 0x00000001;
pub const TS_RAIL_CLIENTSTATUS_AUTORECONNECT: u32 = 0x00000002;
pub const TS_RAIL_CLIENTSTATUS_ZORDER_SYNC: u32 = 0x00000004;
pub const TS_RAIL_CLIENTSTATUS_WINDOW_RESIZE_MARGIN_SUPPORTED: u32 = 0x00000010;
pub const TS_RAIL_CLIENTSTATUS_HIGH_DPI_ICONS_SUPPORTED: u32 = 0x00000020;
pub const TS_RAIL_CLIENTSTATUS_APPBAR_REMOTING_SUPPORTED: u32 = 0x00000040;
pub const TS_RAIL_CLIENTSTATUS_POWER_DISPLAY_REQUEST_SUPPORTED: u32 = 0x00000080;
pub const TS_RAIL_CLIENTSTATUS_GET_APPID_RESPONSE_EX_SUPPORTED: u32 = 0x00000100;
pub const TS_RAIL_CLIENTSTATUS_BIDIRECTIONAL_CLOAK_SUPPORTED: u32 = 0x00000200;

pub struct RailStatusPdu(u32);
impl PduEncode for RailStatusPdu {
    fn encode(&self, dst: &mut WriteCursor<'_>) -> PduResult<()> {
        dst.write_u32(self.0);
        Ok(())
    }

    fn name(&self) -> &'static str {
        return "RailStatusPDU";
    }

    fn size(&self) -> usize {
        return 4; // 四个字节 u32
    }
}
#[derive(Debug)]
pub struct RailExecOrder {
    pub flags: u16,
    pub remote_application_program: String,
    pub remote_application_working_dir: String,
    pub remote_application_arguments: String,
}
impl PduEncode for RailExecOrder {
    fn encode(&self, dst: &mut WriteCursor<'_>) -> PduResult<()> {
        let character_set = CharacterSet::Unicode;
        dst.write_u16(self.flags);
        let program = self.remote_application_program.as_str();
        // let len1 = string_len(program, character_set);
        // debug!("Rail RailExecOrder remote_application_program LEN={}",len1);
        let b = utils::to_utf16_bytes(program);
        let len1 = b.len() as u16;
        dst.write_u16(len1);
        // let len2 = string_len(self.remote_application_working_dir.as_str(), character_set);
        dst.write_u16(0);
        // let len3 = string_len(self.remote_application_arguments.as_str(), character_set);
        dst.write_u16(0);

        // dst.write_u16(program.len() as u16);
        // dst.write_u16(workdir.len() as u16);
        // dst.write_u16(arguments.len() as u16);
        dst.write_slice(b.as_slice());
        // utils::write_string_to_cursor(dst, program, character_set, false)?;
        // utils::write_string_to_cursor(dst, self.remote_application_working_dir.as_str(), character_set, true)?;
        // utils::write_string_to_cursor(dst, self.remote_application_arguments.as_str(), character_set, true)?;

        //
        // dst.write_slice(program.as_bytes());
        Ok(())
    }

    fn name(&self) -> &'static str {
        "RailExecOrder"
    }

    fn size(&self) -> usize {
        let character_set = CharacterSet::Unicode;
        // let nullSize = character_set.to_usize().unwrap();

        let mut legnth: usize = 8;

        // legnth += self.remote_application_program.as_bytes().len();
        let b = utils::to_utf16_bytes(&self.remote_application_program);
        let len1 = b.len();
        // let len1 = string_len(self.remote_application_program.as_str(), character_set);
        legnth += len1 as usize;
        // let len2 = string_len(self.remote_application_working_dir.as_str(), character_set);
        // legnth += len2 as usize;
        // let len3 = string_len(self.remote_application_arguments.as_str(), character_set);
        // legnth += len3 as usize;
        // legnth += nullSize * 1;
        debug!("Rail RailExecOrder LEN={}", legnth);
        legnth
    }
}

pub struct RailPDUHeader {
    pub OrderType: RailOrderType,
    pub OrderLength: u16,

}
impl RailPDUHeader {
    pub const FIXED_PART_SIZE: usize = 4;
}

impl PduEncode for RailPDUHeader {
    fn encode(&self, dst: &mut WriteCursor<'_>) -> PduResult<()> {
        dst.write_u16(self.OrderType.0);
        dst.write_u16(self.OrderLength);
        Ok(())
    }

    fn name(&self) -> &'static str {
        "RailPDUHeader"
    }

    fn size(&self) -> usize {
        Self::FIXED_PART_SIZE
    }
}
impl PduDecode<'_> for RailPDUHeader {
    fn decode(src: &mut ReadCursor<'_>) -> PduResult<Self> {
        let msg_type = src.read_u16();
        let len = src.read_u16();
        Ok(RailPDUHeader {
            OrderType: RailOrderType(msg_type),
            OrderLength: len,
        })
    }
}

fn string_len(value: &str, character_set: CharacterSet) -> u16 {
    match character_set {
        CharacterSet::Ansi => u16::try_from(value.len()).unwrap(),
        CharacterSet::Unicode => u16::try_from(value.encode_utf16().count() * 2).unwrap(),
    }
}