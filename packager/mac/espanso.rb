# Documentation: https://docs.brew.sh/Formula-Cookbook
#                https://rubydoc.brew.sh/Formula
# PLEASE REMOVE ALL GENERATED COMMENTS BEFORE SUBMITTING YOUR PULL REQUEST!
class Espanso < Formula
  desc "{{{app_desc}}}"
  homepage "{{{app_url}}}"
  url "https://github.com/federico-terzi/espanso/releases/download/v{{{app_version}}}/espanso-mac.tar.gz"
  sha256 "{{{release_hash}}}"
  version "{{{app_version}}}"

  resource "modulo" do
    url "https://github.com/federico-terzi/modulo/releases/download/v{{{modulo_version}}}/modulo-mac"
    sha256 "{{{modulo_sha}}}"
  end

  def install
    bin.install "espanso"

    resource("modulo").stage { bin.install "modulo-mac" => "modulo" }
  end
end
