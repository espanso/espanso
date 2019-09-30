# Documentation: https://docs.brew.sh/Formula-Cookbook
#                https://rubydoc.brew.sh/Formula
# PLEASE REMOVE ALL GENERATED COMMENTS BEFORE SUBMITTING YOUR PULL REQUEST!
class Espanso < Formula
  desc "{{{app_desc}}}"
  homepage "{{{app_url}}}"
  url "https://github.com/federico-terzi/espanso/releases/latest/download/espanso-mac.tar.gz"
  sha256 "{{{release_hash}}}"
  version "{{{app_version}}}"
  depends_on "openssl@1.1"

  def install
    bin.install "espanso"
  end
end