#!/bin/env bash
mkdir /tmp/$1

GROUP=`mysql -u mozilliansprodu -p$SQL_PW -h $SQL_HOST mozilliansproddb -e "select id from groups_group where name = \"$1\";" | tail -1`
mysql -u mozilliansprodu -p$SQL_PW -h $SQL_HOST mozilliansproddb -e "select name, ifnull(invalidation_days, 0) as expiration, ifnull(terms, \"\") as terms, description, ifnull(invite_email_text, \"\") as invitation_email, ifnull(accepting_new_members, \"\") as typ, website, wiki from groups_group where name = \"$1\";" | sed 's/\r//g' > /tmp/$1/g.tsv
mysql -u mozilliansprodu -p$SQL_PW -h $SQL_HOST mozilliansproddb -e "select p.auth0_user_id from groups_group_curators as c join profile as p on c.userprofile_id = p.id  where group_id=${GROUP};" | sed 's/\r//g' > /tmp/$1/c.tsv
mysql -u mozilliansprodu -p$SQL_PW -h $SQL_HOST mozilliansproddb -e "select m.date_joined, m.updated_on, p.auth0_user_id, ifnull(g.invalidation_days, 0) as expiration, ifnull(h.auth0_user_id, \"\") as host from groups_groupmembership as m join groups_group as g on g.id = m.group_id join users_idpprofile as p on m.userprofile_id=p.profile_id left join (select pi.auth0_user_id, i.redeemer_id from profile as pi join groups_invite as i on pi.id = i.inviter_id where i.group_id=$GROUP) as h on h.redeemer_id=m.userprofile_id where m.status=\"member\" AND g.id=$GROUP;" | sed 's/\r//g' > /tmp/$1/m.tsv
