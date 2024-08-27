package cfg

import (
	"fmt"
	"os"
	"strings"

	"github.com/google/uuid"

	"github.com/golemcloud/golem-go/binding"
	"github.com/golemcloud/golem-go/golemhost"
)

func ComponentIDFromEnv(key string) (golemhost.ComponentID, error) {
	value := os.Getenv(key)
	if value == "" {
		return [16]byte{}, fmt.Errorf("missing environment variable for component id: %s", key)
	}
	componentID, err := uuid.Parse(strings.ToLower(value))
	if err != nil {
		return [16]byte{}, fmt.Errorf("component id parse failed for %s=%s, %w", key, value, err)
	}
	return golemhost.ComponentID(componentID), nil
}

func ComponentOneID() (golemhost.ComponentID, error) {
	return ComponentIDFromEnv("COMPONENT_ONE_ID")
}

func ComponentTwoID() (golemhost.ComponentID, error) {
	return ComponentIDFromEnv("COMPONENT_TWO_ID")
}

func ComponentThreeID() (golemhost.ComponentID, error) {
	return ComponentIDFromEnv("COMPONENT_THREE_ID")
}

func ComponentOneWorkerURI(workerName string) (binding.GolemRpc0_1_0_TypesUri, error) {
	uri, err := workerURIF(ComponentOneID, workerName)
	return uri, err
}

func ComponentTwoWorkerURI(workerName string) (binding.GolemRpc0_1_0_TypesUri, error) {
	uri, err := workerURIF(ComponentTwoID, workerName)
	return uri, err
}

func ComponentThreeWorkerURI(workerName string) (binding.GolemRpc0_1_0_TypesUri, error) {
	uri, err := workerURIF(ComponentThreeID, workerName)
	return uri, err
}

func WorkerURI[T binding.GolemRpc0_1_0_TypesUri](workerID golemhost.WorkerID) T {
	return T(binding.GolemRpc0_1_0_TypesUri{
		Value: fmt.Sprintf("urn:worker:%s/%s", (uuid.UUID(workerID.ComponentID)).URN(), workerID.WorkerName),
	})
}

func workerURIF(getComponentID func() (golemhost.ComponentID, error), workerName string) (binding.GolemRpc0_1_0_TypesUri, error) {
	componentID, err := getComponentID()
	if err != nil {
		return binding.GolemRpc0_1_0_TypesUri{}, err
	}
	return WorkerURI(golemhost.WorkerID{
		ComponentID: componentID,
		WorkerName:  workerName,
	}), nil
}
