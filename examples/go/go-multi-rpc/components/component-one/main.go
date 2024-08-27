package main

import (
	"fmt"

	"github.com/golemcloud/golem-go/golemhost"
	"github.com/golemcloud/golem-go/std"

	"golem-go-project/components/component-one/binding"
	// NOTE: use the lib folder to create common packages used by multiple components
	"golem-go-project/lib/cfg"
)

func init() {
	binding.SetExportsPackNsComponentOneComponentOneApi(&Impl{})
}

type Impl struct {
	counter uint64
}

func (i *Impl) Add(value uint64) {
	std.Init(std.Packages{Os: true, NetHttp: true})

	selfWorkerName := golemhost.GetSelfMetadata().WorkerId.WorkerName

	{
		componentTwoWorkerURI, err := cfg.ComponentTwoWorkerURI(selfWorkerName)
		if err != nil {
			fmt.Printf("%+v\n", err)
			return
		}

		fmt.Printf("Calling %s...\n", componentTwoWorkerURI.Value)
		componentTwo := binding.NewComponentTwoApi(binding.GolemRpc0_1_0_TypesUri(componentTwoWorkerURI))
		defer componentTwo.Drop()
		componentTwo.BlockingAdd(value)
	}

	{
		componentThreeWorkerURI, err := cfg.ComponentThreeWorkerURI(selfWorkerName)
		if err != nil {
			fmt.Printf("%+v\n", err)
			return
		}

		fmt.Printf("Calling %s...\n", componentThreeWorkerURI.Value)
		componentThree := binding.NewComponentThreeApi(binding.GolemRpc0_1_0_TypesUri(componentThreeWorkerURI))
		defer componentThree.Drop()
		componentThree.BlockingAdd(value)
	}

	i.counter += value
}

func (i *Impl) Get() uint64 {
	std.Init(std.Packages{Os: true, NetHttp: true})

	return i.counter
}

func main() {}
