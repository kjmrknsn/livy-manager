use config::LDAP;
use ldap3::{LdapConn, Scope};
use server::UserSession;
use std::error::Error;

pub fn auth(conf: &LDAP, uid: &str, password: &str) -> Result<UserSession, Box<Error>> {
    let ldap = LdapConn::new(conf.url.as_str())?;

    let user_dn = conf.user_dn.replace("{}", uid);

    ldap.simple_bind(user_dn.as_str(), password)?.success()?;

    let (res, _) = ldap.search(conf.admin_group_dn.as_str(), Scope::Subtree, format!("member={}", user_dn).as_str(), Vec::<&'static str>::new())?.success()?;

    let is_admin = res.len() == 1;

    let uid = String::from(uid);

    Ok(UserSession {
        uid,
        is_admin
    })
}
