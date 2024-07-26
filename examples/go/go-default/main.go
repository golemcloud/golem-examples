package main

import (
	"github.com/golemcloud/golem-go/roundtrip"
	"pack/name/component_name"

	"net/http"
)

type RequestBody struct {
	CurrentTotal uint64
}

type ResponseBody struct {
	Message string
}

func init() {
	a := ComponentNameImpl{}
	component_name.SetExportsPackNameApi(a)
	http.DefaultClient.Transport = roundtrip.WasiHttpTransport{}
}

// total State can be stored in global variables
var total uint64

type ComponentNameImpl struct {
}

// Implementation of the exported interface

func (e ComponentNameImpl) Add(value uint64) {
	total += value
}

func (e ComponentNameImpl) Get() uint64 {
	return total
}

func main() {
}
