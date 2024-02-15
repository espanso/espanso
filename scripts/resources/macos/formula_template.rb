cask "espanso" do
  version "{{{VERSION}}}"

  if Hardware::CPU.intel?
    url "https://github.com/espanso/espanso/releases/download/v#{version}/Espanso-Mac-Intel.zip"
    sha256 "{{{INTEL_SHA}}}"
  else
    url "https://github.com/espanso/espanso/releases/download/v#{version}/Espanso-Mac-M1.zip"
    sha256 "{{{M1_SHA}}}"
  end

  name "Espanso"
  desc "A Privacy-first, Cross-platform Text Expander"
  homepage "https://espanso.org/"

  app "Espanso.app"

  zap trash: "~/Library/Caches/espanso"
end