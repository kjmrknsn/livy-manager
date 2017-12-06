use config::LDAP;
use ldap3::{LdapConn, Scope};
use server::UserSession;
use std::error::Error;

/// LDAP client
#[derive(Clone)]
pub struct Client {
    conf: LDAP,
}

impl Client {
    /// Creates a new LDAP client.
    pub fn new(conf: LDAP) -> Client {
        Client {
            conf,
        }
    }

    pub fn auth(&self, uid: &str, password: &str) -> Result<UserSession, Box<Error>> {
        let ldap = LdapConn::new(self.conf.url.as_str())?;

        let user_dn = self.conf.user_dn.replace("{}", uid);

        ldap.simple_bind(user_dn.as_str(), password)?.success()?;

        let (res, _) = ldap.search(self.conf.admin_group_dn.as_str(), Scope::Subtree, format!("member={}", user_dn).as_str(), Vec::<&'static str>::new())?.success()?;

        let is_admin = res.len() == 1;

        let uid = String::from(uid);

        Ok(UserSession {
            uid,
            is_admin
        })
    }
}
