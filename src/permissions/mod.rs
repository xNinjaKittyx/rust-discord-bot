use crate::{Context, Error, KV_DATABASE, PERMISSIONS};
use poise::serenity_prelude as serenity;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Permission {
    Admin,
    Mod,
    Trusted,
}

impl Permission {
    pub fn as_str(&self) -> &'static str {
        match self {
            Permission::Admin => "admin",
            Permission::Mod => "mod",
            Permission::Trusted => "trusted",
        }
    }
}

impl std::str::FromStr for Permission {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "admin" => Ok(Permission::Admin),
            "mod" => Ok(Permission::Mod),
            "trusted" => Ok(Permission::Trusted),
            _ => Err(format!("Invalid permission: {}", s)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct UserPermissions {
    user_id: u64,
    permissions: Vec<Permission>,
}

// Helper function to check if a user has permission
pub async fn has_permission(user_id: u64, permission: Permission) -> Result<bool, Error> {
    let db = KV_DATABASE.get().unwrap();
    let tx = db.begin_read()?;
    let table = tx.open_table(PERMISSIONS)?;

    let key = user_id.to_string();
    if let Some(value) = table.get(key.as_str())? {
        let perms: UserPermissions = serde_json::from_str(value.value())?;
        Ok(perms.permissions.contains(&permission))
    } else {
        Ok(false)
    }
}

// Helper function to check if user is admin or AUTHOR_ID
async fn is_admin_or_author(ctx: Context<'_>) -> Result<bool, Error> {
    let user_id = ctx.author().id.get();
    if user_id == *crate::env::AUTHOR_ID {
        return Ok(true);
    }
    has_permission(user_id, Permission::Admin).await
}

// Check function for poise commands - must return Result<bool, Error>
pub async fn check_admin(ctx: Context<'_>) -> Result<bool, Error> {
    is_admin_or_author(ctx).await
}

// Check function for moderator permission
pub async fn check_mod(ctx: Context<'_>) -> Result<bool, Error> {
    let user_id = ctx.author().id.get();
    if user_id == *crate::env::AUTHOR_ID {
        return Ok(true);
    }
    Ok(has_permission(user_id, Permission::Admin).await? || has_permission(user_id, Permission::Mod).await?)
}

// Check function for trusted permission
pub async fn check_trusted(ctx: Context<'_>) -> Result<bool, Error> {
    let user_id = ctx.author().id.get();
    if user_id == *crate::env::AUTHOR_ID {
        return Ok(true);
    }
    Ok(has_permission(user_id, Permission::Admin).await?
        || has_permission(user_id, Permission::Mod).await?
        || has_permission(user_id, Permission::Trusted).await?)
}

#[poise::command(prefix_command, slash_command, check = "check_admin", category = "Permissions")]
pub async fn addperm(
    ctx: Context<'_>,
    #[description = "User to grant permission"] user: serenity::User,
    #[description = "Permission level (admin, mod, trusted)"] permission: String,
) -> Result<(), Error> {
    let perm = match permission.parse::<Permission>() {
        Ok(p) => p,
        Err(_) => {
            ctx.say("❌ Invalid permission level. Use: admin, mod, or trusted").await?;
            return Ok(());
        }
    };

    let user_id = user.id.get();
    let db = KV_DATABASE.get().unwrap();

    // Read current permissions
    let mut perms = {
        let tx = db.begin_read()?;
        let table = tx.open_table(PERMISSIONS)?;
        let key = user_id.to_string();

        if let Some(value) = table.get(key.as_str())? {
            serde_json::from_str::<UserPermissions>(value.value())?
        } else {
            UserPermissions {
                user_id,
                permissions: Vec::new(),
            }
        }
    };

    // Add permission if not already present
    if !perms.permissions.contains(&perm) {
        perms.permissions.push(perm.clone());

        // Write back to database
        let tx = db.begin_write()?;
        {
            let mut table = tx.open_table(PERMISSIONS)?;
            let key = user_id.to_string();
            let value = serde_json::to_string(&perms)?;
            table.insert(key.as_str(), value.as_str())?;
        }
        tx.commit()?;

        ctx.say(format!(
            "✅ Granted **{}** permission to <@{}>",
            perm.as_str(), user.id
        ))
        .await?;
    } else {
        ctx.say(format!(
            "ℹ️ <@{}> already has **{}** permission",
            user.id,
            perm.as_str()
        ))
        .await?;
    }

    Ok(())
}

#[poise::command(prefix_command, slash_command, check = "check_admin", category = "Permissions")]
pub async fn removeperm(
    ctx: Context<'_>,
    #[description = "User to revoke permission from"] user: serenity::User,
    #[description = "Permission level (admin, mod, trusted)"] permission: String,
) -> Result<(), Error> {
    let perm = match permission.parse::<Permission>() {
        Ok(p) => p,
        Err(_) => {
            ctx.say("❌ Invalid permission level. Use: admin, mod, or trusted").await?;
            return Ok(());
        }
    };

    let user_id = user.id.get();
    let db = KV_DATABASE.get().unwrap();

    // Read current permissions
    let tx_read = db.begin_read()?;
    let table_read = tx_read.open_table(PERMISSIONS)?;
    let key = user_id.to_string();

    if let Some(value) = table_read.get(key.as_str())? {
        let mut perms: UserPermissions = serde_json::from_str(value.value())?;
        drop(table_read);
        drop(tx_read);

        if let Some(pos) = perms.permissions.iter().position(|p| p == &perm) {
            perms.permissions.remove(pos);

            // Write back to database
            let tx = db.begin_write()?;
            {
                let mut table = tx.open_table(PERMISSIONS)?;
                if perms.permissions.is_empty() {
                    // Remove entry if no permissions left
                    table.remove(key.as_str())?;
                } else {
                    let value = serde_json::to_string(&perms)?;
                    table.insert(key.as_str(), value.as_str())?;
                }
            }
            tx.commit()?;

            ctx.say(format!(
                "✅ Revoked **{}** permission from <@{}>",
                perm.as_str(), user.id
            ))
            .await?;
        } else {
            ctx.say(format!(
                "ℹ️ <@{}> doesn't have **{}** permission",
                user.id,
                perm.as_str()
            ))
            .await?;
        }
    } else {
        ctx.say(format!("ℹ️ <@{}> has no permissions", user.id))
            .await?;
    }

    Ok(())
}

#[poise::command(prefix_command, slash_command, category = "Permissions")]
pub async fn listperms(ctx: Context<'_>) -> Result<(), Error> {
    let db = KV_DATABASE.get().unwrap();
    let tx = db.begin_read()?;
    let table = tx.open_table(PERMISSIONS)?;

    let mut all_perms: Vec<UserPermissions> = Vec::new();
    for item in table.range::<&str>(..)? {
        let (_, value) = item?;
        if let Ok(perms) = serde_json::from_str::<UserPermissions>(value.value()) {
            all_perms.push(perms);
        }
    }

    if all_perms.is_empty() {
        ctx.say("No permissions have been assigned.").await?;
        return Ok(());
    }

    // Group by permission type
    let mut admins = Vec::new();
    let mut mods = Vec::new();
    let mut trusted = Vec::new();

    for perm in all_perms {
        if perm.permissions.contains(&Permission::Admin) {
            admins.push(perm.user_id);
        }
        if perm.permissions.contains(&Permission::Mod) {
            mods.push(perm.user_id);
        }
        if perm.permissions.contains(&Permission::Trusted) {
            trusted.push(perm.user_id);
        }
    }

    let mut response = String::from("**Permissions:**\n");

    if !admins.is_empty() {
        response.push_str("\n**Admins:**\n");
        for user_id in admins {
            response.push_str(&format!("• <@{}>\n", user_id));
        }
    }

    if !mods.is_empty() {
        response.push_str("\n**Moderators:**\n");
        for user_id in mods {
            response.push_str(&format!("• <@{}>\n", user_id));
        }
    }

    if !trusted.is_empty() {
        response.push_str("\n**Trusted:**\n");
        for user_id in trusted {
            response.push_str(&format!("• <@{}>\n", user_id));
        }
    }

    ctx.say(response).await?;
    Ok(())
}
