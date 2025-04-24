#[derive(PartialEq)]
pub enum Message {
    LeftSpeed(f32),
    RightSpeed(f32),
    Stop,
}
