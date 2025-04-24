use core::fmt;

use super::{
    comands::{
        STOP, EQ_VAL, LEFT_SPEED_PREFIX, RIGHT_SPEED_PREFIX, SEPPARATOR,
    },
    message::Message,
};

#[derive(PartialEq)]
pub enum ParsingError {
    NoSepparator,
    ValueCanNotBeParsed,
    NotAComand,
}

impl fmt::Display for ParsingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Can't parse the message")
    }
}

pub fn parse(signal: &str) -> Result<Message, ParsingError> {
    let Some(sep_idx) = get_sepparator_index(signal, SEPPARATOR) else {
        return Err(ParsingError::NoSepparator);
    };
    let cut_part_1 = &signal[..sep_idx];

    let Some(val_sep_idx) = get_sepparator_index(cut_part_1, EQ_VAL) else {
        return Err(ParsingError::NoSepparator);
    };

    let comand = &cut_part_1[..val_sep_idx];
    let value = &cut_part_1[val_sep_idx + 1..];

    match comand {
        LEFT_SPEED_PREFIX => {
            if let Ok(speed) = value.parse::<f32>() {
                Ok(Message::LeftSpeed(speed))
            } else {
                Err(ParsingError::ValueCanNotBeParsed)
            }
        }

        RIGHT_SPEED_PREFIX => {
            if let Ok(speed) = value.parse::<f32>() {
                Ok(Message::RightSpeed(speed))
            } else {
                Err(ParsingError::ValueCanNotBeParsed)
            }
        }
        STOP => Ok(Message::Stop),
        _ => Err(ParsingError::NotAComand),
    }
}

fn get_sepparator_index(string: &str, sepparator: char) -> Option<usize> {
    let mut sep_index = None;
    for (i, ch) in string.chars().enumerate() {
        if ch == sepparator {
            sep_index = Some(i);
            break;
        }
    }
    sep_index
}
