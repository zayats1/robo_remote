use embedded_hal::pwm::SetDutyCycle;
use log::debug;

#[derive(Debug, Default, Clone, Copy)]
pub enum Direction {
    #[default]
    Forward,
    Backward,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Motor<T: SetDutyCycle, U: SetDutyCycle> {
    forward_pin: T,
    backward_pin: U,
    speed: i16,
    direction: Direction,
}

impl<T, U> Motor<T, U>
where
    T: SetDutyCycle,
    U: SetDutyCycle,
{
    pub fn new(forward_pin: T, backward_pin: U) -> Self {
        Self {
            forward_pin,
            backward_pin,
            speed:0,
            direction:Direction::default(),
        }
    }

    pub fn set_dir(&mut self, dir: Direction) {
        debug!("direction = {:?}", self.direction);
        self.direction = dir;
    }

    pub fn get_dir(&mut self) -> Direction {
        self.direction
    }

    // Todo suppot for negative speed(invert dirrection)
    pub fn get_speed(&self) -> i16 {
        self.speed
    }

    pub fn stop(&mut self) {
        debug!("stop");
        self.speed = 0;
        let _ = self.forward_pin.set_duty_cycle_fully_off();
        let _ = self.backward_pin.set_duty_cycle_fully_off();
    }

    pub fn run(&mut self, speed: i16) {
        if speed < 0 {
            self.speed = -speed;
            self.set_dir(Direction::Backward);
        } else {
            self.set_dir(Direction::Forward);
            self.speed = speed;
        }

        if self.speed > 100 {
            self.speed = 100;
        }

        let (_, _) = match self.direction {
            Direction::Forward => (
                self.forward_pin.set_duty_cycle_percent(self.speed as u8),
                self.backward_pin.set_duty_cycle_percent(0),
            ),
            Direction::Backward => (
                self.forward_pin.set_duty_cycle_percent(0),
                self.backward_pin.set_duty_cycle_percent(self.speed as u8),
            ),
        };
    }
}
