use crate::db::internal;
use crate::db::logs::log_comment_body;
use crate::db::logs::LogContext;
use crate::db::model::*;
use crate::db::operations::models::DisplayInvitation;
use crate::db::operations::models::InvitationAndHost;
use crate::db::schema;
use crate::db::schema::groups::dsl as groups;
use crate::db::types::LogOperationType;
use crate::db::types::LogTargetType;
use crate::db::types::TrustType;
use crate::db::views;

use crate::user::User;
use crate::utils::to_expiration_ts;
use chrono::NaiveDateTime;
use diesel::dsl::count;
use diesel::prelude::*;
use failure::Error;

macro_rules! scoped_invitations_for_user {
    ($t:ident, $h:ident, $f:ident) => {
        pub fn $f(connection: &PgConnection, user: &User) -> Result<Vec<DisplayInvitation>, Error> {
            use schema::groups as g;
            use schema::invitations as i;
            use schema::terms as t;
            use schema::$t as u;
            use views::$h as h;
            i::table
                .filter(i::user_uuid.eq(user.user_uuid))
                .inner_join(g::table.on(g::group_id.eq(i::group_id)))
                .filter(g::active.eq(true))
                .left_outer_join(t::table.on(t::group_id.eq(i::group_id)))
                .inner_join(u::table.on(u::user_uuid.eq(i::user_uuid)))
                .inner_join(h::table.on(h::user_uuid.eq(i::added_by)))
                .select((
                    u::user_uuid,
                    u::picture,
                    u::first_name,
                    u::last_name,
                    u::username,
                    u::email,
                    u::trust.eq(TrustType::Staff),
                    i::invitation_expiration,
                    i::group_expiration,
                    g::name,
                    t::text.is_not_null(),
                    h::user_uuid,
                    h::first_name,
                    h::last_name,
                    h::username,
                    h::email,
                ))
                .get_results::<InvitationAndHost>(connection)
                .map(|invitations| invitations.into_iter().map(|m| m.into()).collect())
                .map_err(Into::into)
        }
    };
}

macro_rules! scoped_invitations_for {
    ($t:ident, $h:ident, $f:ident) => {
        pub fn $f(
            connection: &PgConnection,
            group_name: &str,
        ) -> Result<Vec<DisplayInvitation>, Error> {
            use schema::groups as g;
            use schema::invitations as i;
            use schema::terms as t;
            use schema::$t as u;
            use views::$h as h;
            g::table
                .filter(g::name.eq(group_name))
                .filter(g::active.eq(true))
                .inner_join(i::table.on(i::group_id.eq(g::group_id)))
                .left_outer_join(t::table.on(t::group_id.eq(i::group_id)))
                .inner_join(u::table.on(u::user_uuid.eq(i::user_uuid)))
                .inner_join(h::table.on(h::user_uuid.eq(i::added_by)))
                .select((
                    u::user_uuid,
                    u::picture,
                    u::first_name,
                    u::last_name,
                    u::username,
                    u::email,
                    u::trust.eq(TrustType::Staff),
                    i::invitation_expiration,
                    i::group_expiration,
                    g::name,
                    t::text.is_not_null(),
                    h::user_uuid,
                    h::first_name,
                    h::last_name,
                    h::username,
                    h::email,
                ))
                .get_results::<InvitationAndHost>(connection)
                .map(|invitations| invitations.into_iter().map(|m| m.into()).collect())
                .map_err(Into::into)
        }
    };
}

scoped_invitations_for!(users_staff, hosts_staff, staff_scoped_invitations_and_host);
scoped_invitations_for!(users_ndaed, hosts_ndaed, ndaed_scoped_invitations_and_host);
scoped_invitations_for!(
    users_vouched,
    hosts_vouched,
    vouched_scoped_invitations_and_host
);
scoped_invitations_for!(
    users_authenticated,
    hosts_authenticated,
    authenticated_scoped_invitations_and_host
);
scoped_invitations_for!(
    users_public,
    hosts_public,
    public_scoped_invitations_and_host
);

scoped_invitations_for_user!(
    users_staff,
    hosts_staff,
    staff_scoped_invitations_and_host_for_user
);
scoped_invitations_for_user!(
    users_ndaed,
    hosts_ndaed,
    ndaed_scoped_invitations_and_host_for_user
);
scoped_invitations_for_user!(
    users_vouched,
    hosts_vouched,
    vouched_scoped_invitations_and_host_for_user
);
scoped_invitations_for_user!(
    users_authenticated,
    hosts_authenticated,
    authenticated_scoped_invitations_and_host_for_user
);
scoped_invitations_for_user!(
    users_public,
    hosts_public,
    public_scoped_invitations_and_host_for_user
);

pub fn update(
    connection: &PgConnection,
    group_name: &str,
    host: User,
    member: User,
    invitation_expiration: Option<NaiveDateTime>,
    group_expiration: Option<i32>,
) -> Result<(), Error> {
    let group = internal::group::get_group(connection, group_name)?;
    let log_ctx = LogContext::with(group.id, host.user_uuid).with_user(member.user_uuid);
    diesel::update(schema::invitations::table)
        .filter(schema::invitations::user_uuid.eq(member.user_uuid))
        .filter(schema::invitations::group_id.eq(group.id))
        .set((
            invitation_expiration.map(|e| schema::invitations::invitation_expiration.eq(e)),
            (group_expiration.map(|e| schema::invitations::group_expiration.eq(e))),
        ))
        .execute(&*connection)
        .map(|_| {
            internal::log::db_log(
                connection,
                &log_ctx,
                LogTargetType::Invitation,
                LogOperationType::Updated,
                None,
            );
        })
        .map_err(Error::from)
}

pub fn delete(
    connection: &PgConnection,
    group_name: &str,
    host: User,
    member: User,
) -> Result<(), Error> {
    let group = internal::group::get_group(connection, group_name)?;
    let log_ctx = LogContext::with(group.id, host.user_uuid).with_user(member.user_uuid);
    diesel::delete(schema::invitations::table)
        .filter(schema::invitations::user_uuid.eq(member.user_uuid))
        .filter(schema::invitations::group_id.eq(group.id))
        .execute(&*connection)
        .map(|_| {
            internal::log::db_log(
                connection,
                &log_ctx,
                LogTargetType::Invitation,
                LogOperationType::Deleted,
                None,
            );
        })
        .map_err(Error::from)
}

pub fn invite(
    connection: &PgConnection,
    group_name: &str,
    host: User,
    member: User,
    invitation_expiration: Option<NaiveDateTime>,
    group_expiration: Option<i32>,
) -> Result<(), Error> {
    let group = internal::group::get_group(connection, group_name)?;
    let invitation = Invitation {
        user_uuid: member.user_uuid,
        group_id: group.id,
        invitation_expiration,
        group_expiration,
        added_by: host.user_uuid,
    };
    let log_ctx = LogContext::with(group.id, host.user_uuid).with_user(member.user_uuid);
    diesel::insert_into(schema::invitations::table)
        .values(&invitation)
        .execute(&*connection)
        .map(|_| {
            internal::log::db_log(
                connection,
                &log_ctx,
                LogTargetType::Invitation,
                LogOperationType::Created,
                None,
            );
        })
        .map_err(Error::from)
}

pub fn pending_count(connection: &PgConnection, group_name: &str) -> Result<i64, Error> {
    let count = schema::invitations::table
        .inner_join(groups::groups)
        .filter(groups::name.eq(group_name))
        .select(count(schema::invitations::user_uuid))
        .first(connection)?;
    Ok(count)
}

pub fn accept(connection: &PgConnection, group_name: &str, member: &User) -> Result<(), Error> {
    let group = internal::group::get_group(connection, group_name)?;
    let invitation = schema::invitations::table
        .filter(
            schema::invitations::user_uuid
                .eq(member.user_uuid)
                .and(schema::invitations::group_id.eq(group.id)),
        )
        .first::<Invitation>(connection)?;
    let expiration = match invitation.group_expiration {
        Some(exp) => Some(exp),
        None => group.group_expiration,
    }
    .map(to_expiration_ts);
    let role = internal::member::member_role(connection, group_name)?;
    let membership = InsertMembership {
        group_id: invitation.group_id,
        user_uuid: invitation.user_uuid,
        role_id: role.id,
        expiration,
        added_by: invitation.added_by,
    };
    let log_ctx = LogContext::with(group.id, invitation.added_by).with_user(invitation.user_uuid);
    diesel::insert_into(schema::memberships::table)
        .values(&membership)
        .on_conflict((
            schema::memberships::user_uuid,
            schema::memberships::group_id,
        ))
        .do_update()
        .set(&membership)
        .execute(&*connection)
        .map(|_| {
            internal::log::db_log(
                connection,
                &log_ctx,
                LogTargetType::Membership,
                LogOperationType::Created,
                log_comment_body("accepted invitation"),
            );
        })?;
    diesel::delete(schema::invitations::table)
        .filter(
            schema::invitations::user_uuid
                .eq(member.user_uuid)
                .and(schema::invitations::group_id.eq(group.id)),
        )
        .execute(connection)?;
    Ok(())
}
