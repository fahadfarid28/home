group "default" {
  targets = ["home-base"]
}

target "home-base" {
  context = "."
  dockerfile = "Dockerfile"
  target = "home-base"
  tags = ["ghcr.io/bearcove/home-base:latest"]
  platforms = ["linux/amd64", "linux/arm64"]
  output = ["type=registry"]
}
