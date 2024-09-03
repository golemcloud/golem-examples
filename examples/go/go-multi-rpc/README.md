# Golem Go Example with Multiple Components and Worker to Worker RPC Communication

## Building
The project uses [magefile](https://magefile.org/) for building. Either install the tool binary `mage`,
or use the __zero install option__: `go run mage.go`. This readme will use the latter.

To see the available commands use:

```shell
go run mage.go
Targets:
  addStubDependency             adds generated and built stub dependency to componentGolemCliAddStubDependency
  build                         alias for BuildAllComponents
  buildAllComponents            builds all components
  buildComponent                builds component by name
  buildStubComponent            builds RPC stub for component
  clean                         cleans the projects
  deploy                        adds or updates all the components with golem-cli\'s default profile
  generateBinding               generates go binding from WIT
  generateNewComponent          generates a new component based on the component-template
  stubCompose                   composes dependencies
  testIntegration               tests the deployed components
  tinyGoBuildComponentBinary    build wasm component binary with tiny go
  updateRpcStubs                builds rpc stub components and adds them as dependency
  wasmToolsComponentEmbed       embeds type info into wasm component with wasm-tools
  wasmToolsComponentNew         create golem component with wasm-tools
```

For building the project for the first time (or after `clean`) use the following commands:

```shell
go run mage.go updateRpcStubs
go run mage.go build
```

After this, using the `build` command is enough, unless there are changes in the RPC dependencies,
in that case `updateRpcStubs` is needed again.

The final components that are usable by golem are placed in the `target/components` folder.

## Deploying and testing the example

In the example 3 simple counter components are defined, which can be familiar from the smaller examples. To showcase the remote calls, the counters `add` functions are connected, apart from increasing their own counter:
 - **component one** delegates the add call to **component two** and **three** too,
 - and **component two** delegates to **component three**.

In both cases the _current worker name_ will be used as _target worker name_ too. 

Apart from _worker name_, remote calls also require the **target components' deployed ID**. For this the example uses environment variables, and uses the `lib/cfg` subpackage (which is shared between the components) to extract it.

The examples assume a configured default `golem-cli` profile, and will use that.

To test, first we have to build the project as seen in the above:

```shell
go run mage.go updateRpcStubs
go run mage.go build
```

Then we can deploy our components with `golem-cli`, for which a wrapper magefile command is provided:

```shell
go run mage.go deploy
```

Once the components are deployed, a simple example integration test suite can be used to test the components.
The tests are in the [/integration/integration_test.go](/integration/integration_test.go) test file, and can be run with:

```shell
go run mage.go testIntegration
```

The `TestDeployed` simply tests if our components metadata is available through `golem-cli component get`.

The `TestCallingAddOnComponentOneCallsToOtherComponents` will:
 - get the _component URNs_ with `golem-cli component get`
 - generates a _random worker name_, so our tests are starting from a clean state
 - adds 1 - 1 worker for component one and component two with the required _environment variables_ containing the other workers' _component ids_
 - then makes various component invocations with `golem-cli worker invoke-and-await` and tests if the counters - after increments -  are holding the right value according to the delegated `add` function calls.

## Adding Components

Use the `generateNewComponent` command to add new components to the project:

```shell
go run mage.go generateNewComponent component-four
```

The above will create a new component in the `components/component-four` directory based on the template at [/component-template/component](/component-template/component).

After adding a new component the `build` command will also include it.

## Using Worker to Worker RPC calls

### Under the hood 

Under the hood the _magefile_ commands below (and for build) use generic `golem-cli stubgen` subcommands:
 - `golem-cli stubgen build` for creating remote call _stub WIT_ definitions and _WASM components_ for the stubs
 - `golem-cli stubgen add-stub-dependency` for adding the _stub WIT_ definitions to a _component's WIT_ dependencies
 - `golem-cli stubgen compose` for _composing_ components with the stub components

### Magefile commands and required manual steps

The dependencies between components are defined in  the [/magefiles/config.go](/magefiles/config.go) build script:

```go
// componentDeps defines the Worker to Worker RPC dependencies
var componentDeps = map[string][]string{
    "component-one": {"component-two", "component-three"},
    "component-two": {"component-three"},
}
```

After changing dependencies the `updateRpcStubs` command can be used to create the necessary stubs:

```shell
go run mage.go updateRpcStubs
```

The command will create stubs for the dependency projects in the ``/target/stub`` directory and will also place the required stub _WIT_ interfaces on the dependant component's `wit/deps` directory.

To actually use the dependencies in a project it also has to be manually imported in the component's world.

E.g. with the above definitions the following import has to be __manually__ added to `/components/component-one/wit/component-one.wit`:

```wit
import pack-ns:component-two-stub;
import pack-ns:component-three-stub;
```

So the component definition should like similar to this:

```wit
package pack-ns:component-one;

// See https://component-model.bytecodealliance.org/design/wit.html for more details about the WIT syntax

interface component-one-api {
  add: func(value: u64);
  get: func() -> u64;
}

world component-one {
  // Golem dependencies
  import golem:api/host@0.2.0;
  import golem:rpc/types@0.1.0;

  // WASI dependencies
  import wasi:blobstore/blobstore;
  // .
  // .
  // .
  // other dependencies
  import wasi:sockets/instance-network@0.2.0;

  // Project Component dependencies
  import pack-ns:component-two-stub;
  import pack-ns:component-three-stub;

  export component-one-api;
}
```

After this `build` (or the `generateBinding`) command can be used to update bindings, which now should include the
required functions for calling other components.

Here's an example that delegates the `Add` call to another component and waits for the result:

```go
import (
	"github.com/golemcloud/golem-go/std"

	"golem-go-project/components/component-one/binding"
)


func (i *Impl) Add(value uint64) {
    std.Init(std.Packages{Os: true, NetHttp: true})
    
    componentTwo := binding.NewComponentTwoApi(binding.GolemRpc0_1_0_TypesUri{Value: "uri"})
    defer componentTwo.Drop()
    componentTwo.BlockingAdd(value)

    i.counter += value
}
```

Once a remote call is in place, the `build` command will also compose the stub components into the caller component.
