package integration

import (
	"fmt"
	"strings"
	"testing"

	"github.com/google/uuid"
	"github.com/magefile/mage/sh"
	"github.com/tidwall/gjson"
)

func TestDeployed(t *testing.T) {
	componentOneURN := mustGetCompURNByCompName(t, "component-one")
	fmt.Printf("component-one: %s\n", componentOneURN)

	componentTwoURN := mustGetCompURNByCompName(t, "component-two")
	fmt.Printf("component-two: %s\n", componentTwoURN)

	componentThreeURN := mustGetCompURNByCompName(t, "component-three")
	fmt.Printf("component-three: %s\n", componentThreeURN)
}

func TestCallingAddOnComponentOneCallsToOtherComponents(t *testing.T) {
	workerName := uuid.New().String()
	fmt.Printf("random worker name for test: %s\n", workerName)

	// Setup
	{
		// Get all component URNs
		componentURNs := mustGetComponentURNs(t)
		fmt.Printf("component urns: %#v\n", componentURNs)

		// Preparing workers with env vars for RPC, so they know the other component IDs
		mustAddComponent(t, "component-one", workerName, componentURNs)
		mustAddComponent(t, "component-two", workerName, componentURNs)
	}

	// Call get on all component and check the counters are 0
	{
		expectCounter(t, "component-one", workerName, 0)
		expectCounter(t, "component-two", workerName, 0)
		expectCounter(t, "component-three", workerName, 0)
	}

	// Invoke add on component-one
	{
		mustInvokeAndAwaitComponent(t, "component-one", workerName, "pack-ns:component-one/component-one-api.{add}", []string{"3"})
	}

	// Call get on all component and check the counters are accumulated on component two and three
	{
		expectCounter(t, "component-one", workerName, 3)
		expectCounter(t, "component-two", workerName, 3)
		expectCounter(t, "component-three", workerName, 6)
	}

	// Invoke add on component-two
	{
		mustInvokeAndAwaitComponent(t, "component-two", workerName, "pack-ns:component-two/component-two-api.{add}", []string{"2"})
	}

	// Call get on all component and check the counters are accumulated on component three
	{
		expectCounter(t, "component-one", workerName, 3)
		expectCounter(t, "component-two", workerName, 5)
		expectCounter(t, "component-three", workerName, 8)
	}

	// Invoke add on component-one again
	{
		mustInvokeAndAwaitComponent(t, "component-one", workerName, "pack-ns:component-one/component-one-api.{add}", []string{"1"})
	}

	// Call get on all component and check the counters are accumulated on component two and three
	{
		expectCounter(t, "component-one", workerName, 4)
		expectCounter(t, "component-two", workerName, 6)
		expectCounter(t, "component-three", workerName, 10)
	}
}

func getCompURNByCompName(compName string) (string, error) {
	output, err := sh.Output(
		"golem-cli", "--format", "json", "component", "get", "--component"+"-name", compName,
	)
	if err != nil {
		return "", fmt.Errorf("getCompURNByCompName for %s: golem-cli failed: %w\n", compName, err)
	}

	componentURN := gjson.Get(output, "componentUrn").String()
	if componentURN == "" {
		return "", fmt.Errorf("missing componentURN in response:\n%s\n", output)
	}

	return componentURN, nil
}

func mustGetCompURNByCompName(t *testing.T, compName string) string {
	name, err := getCompURNByCompName(compName)
	if err != nil {
		t.Fatalf("%+v", err)
	}
	return name
}

type ComponentURNs struct {
	ComponentOne   string
	ComponentTwo   string
	ComponentThree string
}

func mustGetComponentURNs(t *testing.T) ComponentURNs {
	return ComponentURNs{
		ComponentOne:   mustGetCompURNByCompName(t, "component-one"),
		ComponentTwo:   mustGetCompURNByCompName(t, "component-two"),
		ComponentThree: mustGetCompURNByCompName(t, "component-three"),
	}
}

func addComponent(compName, workerName string, componentURNs ComponentURNs) error {
	fmt.Printf("adding component: %s, %s\n", compName, workerName)
	output, err := sh.Output(
		"golem-cli", "worker",
		"--format", "json",
		"add",
		"--component"+"-name", compName,
		"--worker-name", workerName,
		"--env", fmt.Sprintf("COMPONENT_ONE_ID=%s", componentIDFromURN(componentURNs.ComponentOne)),
		"--env", fmt.Sprintf("COMPONENT_TWO_ID=%s", componentIDFromURN(componentURNs.ComponentTwo)),
		"--env", fmt.Sprintf("COMPONENT_THREE_ID=%s", componentIDFromURN(componentURNs.ComponentThree)),
	)
	if err != nil {
		return fmt.Errorf("addComponent for %s, %s: golem-cli failed: %w\n%s", compName, workerName, err, output)
	}
	return nil
}

func mustAddComponent(t *testing.T, compName, workerName string, componentURNs ComponentURNs) {
	err := addComponent(compName, workerName, componentURNs)
	if err != nil {
		t.Fatalf("%+v", err)
	}
}

func invokeAndAwaitComponent(compName, workerName, function string, functionArgs []string) (string, error) {
	fmt.Printf("invoking component: %s, %s, %s, %+v\n", compName, workerName, function, functionArgs)

	cliArgs := []string{
		"--format", "json",
		"worker",
		"invoke-and-await",
		"--component" + "-name", compName,
		"--worker-name", workerName,
		"--function", function,
	}

	for _, arg := range functionArgs {
		cliArgs = append(cliArgs, []string{"--arg", arg}...)
	}

	output, err := sh.Output("golem-cli", cliArgs...)
	if err != nil {
		return "", fmt.Errorf("invokeAndAwaitComponent failed: %w", err)
	}

	fmt.Println(output)

	return output, nil
}

func mustInvokeAndAwaitComponent(t *testing.T, componentURN, workerName, function string, functionArgs []string) string {
	output, err := invokeAndAwaitComponent(componentURN, workerName, function, functionArgs)
	if err != nil {
		t.Fatalf("%+v", err)
	}
	return output
}

func componentIDFromURN(urn string) string {
	return strings.Split(urn, ":")[2]
}

func expectCounter(t *testing.T, compName, workerName string, expected int64) {
	output := mustInvokeAndAwaitComponent(t, compName, workerName, fmt.Sprintf("pack-ns:%s/%s-api.{get}", compName, compName), nil)

	actualValue := gjson.Get(output, "value")
	if !actualValue.Exists() {
		t.Fatalf("Expected counter for %s, %s: %d, actual value is missing", compName, workerName, expected)
	}

	actualArray := actualValue.Array()
	if len(actualArray) != 1 {
		t.Fatalf("Expected counter for %s, %s: %d, actual value tuple has bad number of elements: %s", compName, workerName, expected, actualValue)
	}

	actual := actualArray[0].Int()
	if expected != actual {
		t.Fatalf("Expected counter for %s, %s: %d, actual: %d", compName, workerName, expected, actual)
	}
}
