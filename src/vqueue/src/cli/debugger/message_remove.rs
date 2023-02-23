/*
 * vSMTP mail transfer agent
 * Copyright (C) 2022 viridIT SAS
 *
 * This program is free software: you can redistribute it and/or modify it under
 * the terms of the GNU General Public License as published by the Free Software
 * Foundation, either version 3 of the License, or any later version.
 *
 * This program is distributed in the hope that it will be useful, but WITHOUT
 * ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
 * FOR A PARTICULAR PURPOSE.  See the GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along with
 * this program. If not, see https://www.gnu.org/licenses/.
 *
 */
use crate::{cli::args::Commands, GenericQueueManager, QueueID};
extern crate alloc;

#[allow(clippy::multiple_inherent_impl)]
impl Commands {
    pub(crate) async fn message_remove<
        OUT: std::io::Write + Send + Sync,
        IN: tokio::io::AsyncRead + Send + Sync + core::marker::Unpin,
    >(
        msg_uuid: &uuid::Uuid,
        confirmed: bool,
        queue_manager: alloc::sync::Arc<impl GenericQueueManager + Send + Sync>,
        output: &mut OUT,
        mut input: IN,
    ) -> anyhow::Result<()> {
        match (futures_util::future::join_all(
            <QueueID as strum::IntoEnumIterator>::iter()
                .map(|q| (q, alloc::sync::Arc::clone(&queue_manager)))
                .map(|(q, manager)| async move { (q.clone(), manager.get_ctx(&q, msg_uuid).await) }),
        )
        .await
        .into_iter()
        .find_map(|(q, ctx)| match ctx {
            Ok(_) => Some(q),
            Err(_) => None,
        }), queue_manager.get_msg(msg_uuid).await) {
            (None, Ok(_)) => {
                anyhow::bail!("Message is orphan: exists but no context in the queue!")
            }
            (None, Err(_)) => {
                anyhow::bail!("Message does not exist in any queue!")
            }
            (Some(_), Err(_)) => {
                anyhow::bail!("Message  is orphan: context in the queue but no message!")
            }
            (Some(queue), Ok(_)) => {
                output.write_fmt(format_args!(
                    "Removing message '{msg_uuid}' in queue: '{queue}'\n",
                ))?;

                if !confirmed {
                    output.write_all(b"Confirm ? [y|yes] ")?;
                    output.flush()?;

                    let buf = &mut [0u8; 1];
                    tokio::io::AsyncReadExt::read(&mut input, buf).await?;

                    if buf[0] != b'y' {
                        output.write_all(b"Canceled\n")?;
                        return Ok(());
                    }
                }

                queue_manager.remove_both(&queue, msg_uuid).await?;
                output.write_all(b"File removed\n")?;
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use vsmtp_test::config::{local_ctx, local_msg, local_test};

    #[tokio::test(flavor = "multi_thread")]
    async fn confirmed() {
        let mut output = vec![];
        let input = std::io::Cursor::new(vec![]);

        let config = alloc::sync::Arc::new(local_test());
        let queue_manager = crate::temp::QueueManager::init(config, vec![]).unwrap();

        let mut ctx = local_ctx();
        let msg_uuid = uuid::Uuid::new_v4();
        ctx.mail_from.message_uuid = msg_uuid;

        queue_manager
            .write_both(&QueueID::Working, &ctx, &local_msg())
            .await
            .unwrap();

        Commands::message_remove(
            &msg_uuid,
            true,
            alloc::sync::Arc::clone(&queue_manager),
            &mut output,
            input,
        )
        .await
        .unwrap();

        queue_manager
            .get_both(&QueueID::Working, &msg_uuid)
            .await
            .unwrap_err();

        pretty_assertions::assert_eq!(
            core::str::from_utf8(&output).unwrap(),
            [
                format!("Removing message '{msg_uuid}' in queue: 'working'\n"),
                "File removed\n".to_owned()
            ]
            .concat()
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn not_confirmed() {
        let mut output = vec![];
        let input = std::io::Cursor::new(b"yes\n");

        let config = alloc::sync::Arc::new(local_test());
        let queue_manager = crate::temp::QueueManager::init(config, vec![]).unwrap();

        let mut ctx = local_ctx();
        let msg_uuid = uuid::Uuid::try_parse("00000000-0000-0000-0000-000000000001").unwrap();
        ctx.mail_from.message_uuid = msg_uuid;

        queue_manager
            .write_both(&QueueID::Working, &ctx, &local_msg())
            .await
            .unwrap();

        Commands::message_remove(
            &msg_uuid,
            false,
            alloc::sync::Arc::clone(&queue_manager),
            &mut output,
            input,
        )
        .await
        .unwrap();

        queue_manager
            .get_both(&QueueID::Working, &msg_uuid)
            .await
            .unwrap_err();

        pretty_assertions::assert_eq!(
            core::str::from_utf8(&output).unwrap(),
            [
                "Removing message '00000000-0000-0000-0000-000000000001' in queue: 'working'\n",
                "Confirm ? [y|yes] ",
                "File removed\n"
            ]
            .concat()
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn canceled() {
        let mut output = vec![];
        let input = std::io::Cursor::new(b"no\n");

        let config = alloc::sync::Arc::new(local_test());
        let queue_manager = crate::temp::QueueManager::init(config, vec![]).unwrap();

        let mut ctx = local_ctx();
        let msg_uuid = uuid::Uuid::try_parse("00000000-0000-0000-0000-000000000002").unwrap();
        ctx.mail_from.message_uuid = msg_uuid;

        queue_manager
            .write_both(&QueueID::Working, &ctx, &local_msg())
            .await
            .unwrap();

        Commands::message_remove(
            &msg_uuid,
            false,
            alloc::sync::Arc::clone(&queue_manager),
            &mut output,
            input,
        )
        .await
        .unwrap();

        queue_manager
            .get_both(&QueueID::Working, &msg_uuid)
            .await
            .unwrap();

        pretty_assertions::assert_eq!(
            core::str::from_utf8(&output).unwrap(),
            [
                "Removing message '00000000-0000-0000-0000-000000000002' in queue: 'working'\n",
                "Confirm ? [y|yes] ",
                "Canceled\n"
            ]
            .concat()
        );
    }
}
