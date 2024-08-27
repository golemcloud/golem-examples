package main

import (
	"github.com/golemcloud/golem-go/std"

	"golem-go-project/components/comp-name/binding"
)

func init() {
	binding.SetExportsPckNsCompNameCompNameApi(&Impl{})
}

type Impl struct {
	counter uint64
}

func (i *Impl) Add(value uint64) {
	std.Init(std.Packages{Os: true, NetHttp: true})

	i.counter += value
}

func (i *Impl) Get() uint64 {
	std.Init(std.Packages{Os: true, NetHttp: true})

	return i.counter
}

func main() {}
