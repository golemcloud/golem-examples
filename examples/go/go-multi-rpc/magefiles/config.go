package main

// componentDeps defines the Worker to Worker RPC dependencies
var componentDeps = map[string][]string{
	"component-one": {"component-two", "component-three"},
	"component-two": {"component-three"},
}

var pkgNs = "pack-ns"
var targetDir = "target"
var componentsDir = "components"
var libDir = "lib"
var wasiSnapshotPreview1Adapter = "adapters/tier1/wasi_snapshot_preview1.wasm"
