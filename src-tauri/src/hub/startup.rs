//! How the Hub is started: what the person came for, from the command line.
use crate::lifecycle;
use serde_json::{json, Value};

/// `--install=<product>` (a product installer handed over to the Hub) or `--manage=<product>`. Unknown products and actions are ignored.
pub(crate) fn intent(args: impl Iterator<Item = String>) -> Option<Value> {
    args.filter_map(|arg| ["install", "manage"].into_iter().find_map(|action| arg.strip_prefix(&format!("--{action}=")).map(|p| (action, p.to_string()))))
        .find(|(_, product)| lifecycle::PRODUCTS.contains(&product.as_str()))
        .map(|(action, product)| json!({ "action": action, "product": product }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_known_products_and_actions_become_an_intent() {
        let args = |list: &[&str]| list.iter().map(|s| s.to_string()).collect::<Vec<_>>().into_iter();
        assert_eq!(intent(args(&["hub.exe", "--install=secondbrain"])), Some(json!({ "action": "install", "product": "secondbrain" })));
        assert_eq!(intent(args(&["hub.exe", "--manage=orchestrator"])), Some(json!({ "action": "manage", "product": "orchestrator" })));
        assert_eq!(intent(args(&["hub.exe", "--uninstall=secondbrain"])), None, "removal is the product's own uninstaller's");
        assert_eq!(intent(args(&["hub.exe", "--install=notes"])), None);
    }
}
