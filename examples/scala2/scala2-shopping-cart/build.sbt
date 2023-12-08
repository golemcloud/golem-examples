ThisBuild / version := "0.1.0"

ThisBuild / scalaVersion := "2.13.12"

lazy val root = (project in file("."))
  .settings(
    name := "component-name"
  )
  .enablePlugins(ScalaJSPlugin)
  .settings(
    scalaJSLinkerConfig ~= { _.withModuleKind(ModuleKind.ESModule) },
    Compile / fullLinkJS / scalaJSLinkerOutputDirectory := target.value / "dist"
  )

lazy val witBindgen = taskKey[Unit]("Runs golem-scalajs-wit-bindgen to generate WIT bindings")

witBindgen := {
  import scala.sys.process._

  Seq("bash", "-xc", "golem-scalajs-wit-bindgen -w wit/main.wit -p example").!!
}

lazy val component = taskKey[Unit]("Runs componentize-js on the generated main.js file")

component := {
  import scala.sys.process._

  Seq("bash", "-xc", "npm install").!!
  Seq("bash", "-xc", "npm run build").!!
}

component := (component dependsOn (Compile / fullLinkJS)).value

Compile / sourceGenerators += Def.task {
  import scala.sys.process._

  val file = (Compile / sourceManaged).value / "scala" / "example" / "Api.scala"

  IO.write(
    file,
    Seq("bash", "-xc", "golem-scalajs-wit-bindgen -w wit/main.wit -p example").!!
  )

  Seq(file)
}.taskValue
