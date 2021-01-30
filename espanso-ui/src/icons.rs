#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TrayIcon {
  Normal,
  Disabled,

  // For example, when macOS activates SecureInput
  SystemDisabled,
}
