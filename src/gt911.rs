use std::{
    sync::atomic::AtomicPtr,
    time::{Duration, Instant},
};

pub const GT911_I2C_SLAVE_ADDR: u8 = 0x5D;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
struct lv_point_t {
    pub x: i16,
    pub y: i16,
}

// typedef enum {
//     LV_INDEV_STATE_RELEASED = 0,
//     LV_INDEV_STATE_PRESSED
// } lv_indev_state_t;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
enum lv_indev_state_t {
    #[default]
    LvIndevStateReleased = 0,
    LvIndevStatePressed,
}

// typedef struct {
//     lv_point_t point; /**< For LV_INDEV_TYPE_POINTER the currently pressed point*/
//     uint32_t key;     /**< For LV_INDEV_TYPE_KEYPAD the currently pressed key*/
//     uint32_t btn_id;  /**< For LV_INDEV_TYPE_BUTTON the currently pressed button*/
//     int16_t enc_diff; /**< For LV_INDEV_TYPE_ENCODER number of steps since the previous read*/
//     lv_indev_state_t state; /**< LV_INDEV_STATE_REL or LV_INDEV_STATE_PR*/
//     bool continue_reading;  /**< If set to true, the read callback is invoked again*/
// } lv_indev_data_t;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
struct lv_indev_data_t {
    pub point: lv_point_t,
    pub key: u32,
    pub btn_id: u32,
    pub enc_diff: i16,
    pub state: lv_indev_state_t,
    pub continue_reading: bool,
}

extern "C" {
    pub fn GT911_RST();

    // void gt911_init(uint8_t dev_addr);
    pub fn gt911_init(dev_addr: u8);

    // bool gt911_read(lv_indev_data_t *data);
    fn gt911_read(data: *mut lv_indev_data_t) -> bool;
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct TouchState {
    pub x: i16,
    pub y: i16,
    pub pressed: bool,
}

static LAST_CHECK_TOUCH: AtomicPtr<Instant> = AtomicPtr::new(core::ptr::null_mut());
static LAST_TOUCH: AtomicPtr<TouchState> = AtomicPtr::new(core::ptr::null_mut());

pub fn read_touch() -> Option<TouchState> {
    let last_check = LAST_CHECK_TOUCH.load(core::sync::atomic::Ordering::Relaxed);
    let last_check = if last_check.is_null() {
        let last_check = Instant::now();
        LAST_CHECK_TOUCH.store(
            Box::into_raw(Box::new(last_check)),
            core::sync::atomic::Ordering::Relaxed,
        );
        last_check
    } else {
        unsafe { *last_check }
    };

    if last_check.elapsed() < Duration::from_millis(10) {
        let last_touch = LAST_TOUCH.load(core::sync::atomic::Ordering::Relaxed);
        if last_touch.is_null() {
            return Some(TouchState::default());
        } else {
            return None;
        }
    }

    let mut input = lv_indev_data_t::default();
    unsafe {
        gt911_read(&mut input);
    }
    let ret = TouchState {
        x: input.point.x,
        y: input.point.y,
        pressed: input.state == lv_indev_state_t::LvIndevStatePressed,
    };
    LAST_CHECK_TOUCH.store(
        Box::into_raw(Box::new(Instant::now())),
        core::sync::atomic::Ordering::Relaxed,
    );
    let current = LAST_TOUCH.load(core::sync::atomic::Ordering::Relaxed);
    if current.is_null() || unsafe { *current } != ret {
        LAST_TOUCH.store(
            Box::into_raw(Box::new(ret)),
            core::sync::atomic::Ordering::Relaxed,
        );
        Some(ret)
    } else {
        None
    }
}
