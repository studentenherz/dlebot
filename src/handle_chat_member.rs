use teloxide::prelude::*;

use crate::database::DatabaseHandler;

pub async fn handle_my_chat_member(
    db_handler: DatabaseHandler,
    update: ChatMemberUpdated,
) -> ResponseResult<()> {
    let ChatMemberUpdated {
        from,
        old_chat_member,
        new_chat_member,
        ..
    } = &update;

    let user_id = from.id.0.try_into().unwrap();

    if old_chat_member.is_present() && !new_chat_member.is_present() {
        db_handler.set_in_bot(user_id, false).await;
        db_handler.add_user_left_event(user_id).await;
    } else if !old_chat_member.is_present() && new_chat_member.is_present() {
        db_handler.set_in_bot(user_id, true).await;
        db_handler.add_user_joined_event(user_id).await;
    } else {
        log::warn!("Got weird MyChatMember update: {:?}", update);
    }

    Ok(())
}
