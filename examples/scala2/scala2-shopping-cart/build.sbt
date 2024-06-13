ThisBuild / version := "0.1.0"

ThisBuild / scalaVersion := "2.13.12"

lazy val root = (project in file("."))
  .enablePlugins(WasmComponentPlugin)
  .settings(
    name := "component-name",
    wasmComponentPackageName := "example"
  )
