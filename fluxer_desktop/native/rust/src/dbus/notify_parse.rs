// SPDX-License-Identifier: AGPL-3.0-or-later

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotifyReply {
    pub id: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionInvokedSignal {
    pub id: u32,
    pub action_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClosedSignal {
    pub id: u32,
    pub reason: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError {
    InvalidReply,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FakeArg<'a> {
    U32(u32),
    String(&'a str),
}

pub fn parse_action_invoked_from_args(
    args: &[FakeArg<'_>],
) -> Result<ActionInvokedSignal, ParseError> {
    match args {
        [FakeArg::U32(id), FakeArg::String(action_key), ..] => Ok(ActionInvokedSignal {
            id: *id,
            action_key: (*action_key).to_owned(),
        }),
        _ => Err(ParseError::InvalidReply),
    }
}

pub fn parse_closed_from_args(args: &[FakeArg<'_>]) -> Result<ClosedSignal, ParseError> {
    match args {
        [FakeArg::U32(id), FakeArg::U32(reason), ..] => Ok(ClosedSignal {
            id: *id,
            reason: *reason,
        }),
        _ => Err(ParseError::InvalidReply),
    }
}

pub fn parse_notify_reply_from_args(args: &[FakeArg<'_>]) -> Result<NotifyReply, ParseError> {
    match args {
        [FakeArg::U32(id), ..] => Ok(NotifyReply { id: *id }),
        _ => Err(ParseError::InvalidReply),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notify_reply_valid_u32_yields_id() {
        let reply = parse_notify_reply_from_args(&[FakeArg::U32(42)]).unwrap();
        assert_eq!(42, reply.id);
    }

    #[test]
    fn notify_reply_rejects_empty_body() {
        assert_eq!(
            Err(ParseError::InvalidReply),
            parse_notify_reply_from_args(&[])
        );
    }

    #[test]
    fn notify_reply_rejects_wrong_type() {
        assert_eq!(
            Err(ParseError::InvalidReply),
            parse_notify_reply_from_args(&[FakeArg::String("wrong")])
        );
    }

    #[test]
    fn action_invoked_valid_body() {
        let sig =
            parse_action_invoked_from_args(&[FakeArg::U32(7), FakeArg::String("default")]).unwrap();
        assert_eq!(7, sig.id);
        assert_eq!("default", sig.action_key);
    }

    #[test]
    fn action_invoked_rejects_swapped_types() {
        assert_eq!(
            Err(ParseError::InvalidReply),
            parse_action_invoked_from_args(&[FakeArg::String("x"), FakeArg::String("y")])
        );
    }

    #[test]
    fn notification_closed_id_and_reason() {
        let sig = parse_closed_from_args(&[FakeArg::U32(11), FakeArg::U32(2)]).unwrap();
        assert_eq!(11, sig.id);
        assert_eq!(2, sig.reason);
    }

    #[test]
    fn notification_closed_rejects_too_short() {
        assert_eq!(
            Err(ParseError::InvalidReply),
            parse_closed_from_args(&[FakeArg::U32(11)])
        );
    }
}
