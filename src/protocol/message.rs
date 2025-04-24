#[derive(PartialEq,Debug,Default)]
pub enum Message {
    LeftSpeed(f32),
    RightSpeed(f32),
    #[default]
    Stop,
}
