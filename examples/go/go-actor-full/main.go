package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io/ioutil"
	"pack/name/component_name"
	"pack/name/roundtrip"

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

func (e ComponentNameImpl) Publish() component_name.Result[struct{}, string] {
	http.DefaultClient.Transport = roundtrip.WasiHttpTransport{}
	var result component_name.Result[struct{}, string]

	postBody, _ := json.Marshal(RequestBody{
		CurrentTotal: total,
	})
	resp, err := http.Post("http://localhost:9999/post-example", "application/json", bytes.NewBuffer(postBody))
	if err != nil {
		result.SetErr(fmt.Sprintln(err))
		return result
	}
	body, err := ioutil.ReadAll(resp.Body)
	if err != nil {
		result.SetErr(fmt.Sprintln(err))
		return result
	}

	var response ResponseBody
	err = json.Unmarshal(body, &response)
	if err != nil {
		result.SetErr(fmt.Sprintln(err))
		return result
	}

	fmt.Println(response.Message)

	result.Set(struct{}{})
	return result
}

func (e ComponentNameImpl) Pause() {
	promise := component_name.GolemApi0_2_0_HostGolemCreatePromise()
	component_name.GolemApi0_2_0_HostGolemAwaitPromise(promise)
}

func main() {
}
