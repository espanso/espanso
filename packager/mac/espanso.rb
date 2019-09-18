# Documentation: https://docs.brew.sh/Formula-Cookbook
#                https://rubydoc.brew.sh/Formula
# PLEASE REMOVE ALL GENERATED COMMENTS BEFORE SUBMITTING YOUR PULL REQUEST!
class Espanso < Formula
  desc "{{{app_desc}}}"
  homepage "{{{app_url}}}"
  url "{{{app_release_url}}}"
  sha256 "{{{release_hash}}}"
  version "{{{app_version}}}"

  def install
    bin.install "espanso"
  end
end