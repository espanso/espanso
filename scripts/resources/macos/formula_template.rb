cask "espanso" do
  version "{{{VERSION}}}"

  url "https://github.com/espanso/espanso/releases/download/v#{version}/Espanso-Mac-Universal.zip"
  sha256 "{{{SHA256}}}"

  name "Espanso"
  desc "A Privacy-first, Cross-platform Text Expander"
  homepage "https://espanso.org/"

  app "Espanso.app"

  zap trash: "~/Library/Caches/espanso"
end
