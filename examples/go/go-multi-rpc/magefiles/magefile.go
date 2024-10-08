package main

import (
	"bytes"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"sort"
	"strings"

	"github.com/magefile/mage/sh"
	"github.com/magefile/mage/target"
)

// Build alias for BuildAllComponents
func Build() error {
	return BuildAllComponents()
}

// BuildAllComponents builds all components
func BuildAllComponents() error {
	for _, componentName := range componentNames() {
		err := BuildComponent(componentName)
		if err != nil {
			return fmt.Errorf("build all components: build component failed for %s, %w", componentName, err)
		}
	}

	return nil
}

// UpdateRpcStubs builds rpc stub components and adds them as dependency
func UpdateRpcStubs() error {
	for _, componentName := range stubComponentNames() {
		err := BuildStubComponent(componentName)
		if err != nil {
			return fmt.Errorf("update RPC stubs: build stub component failed for %s, %w", componentName, err)
		}
	}

	for _, componentName := range componentNames() {
		for _, dependency := range componentDeps[componentName] {
			err := AddStubDependency(componentName, dependency)
			if err != nil {
				return fmt.Errorf("update RPC stubs: add stub dependecy failed for %s to %s, %w", dependency, componentName, err)
			}
		}
	}

	return nil
}

// BuildStubComponent builds RPC stub for component
func BuildStubComponent(componentName string) error {
	componentDir := filepath.Join(componentsDir, componentName)
	srcWitDir := filepath.Join(componentDir, "wit")
	stubTargetDir := filepath.Join(targetDir, "stub", componentName)
	destWasm := filepath.Join(stubTargetDir, "stub.wasm")
	destWitDir := filepath.Join(stubTargetDir, "wit")

	return opRun(op{
		RunMessage:  fmt.Sprintf("Building stub component for %s", componentName),
		SkipMessage: "stub component build",
		Targets:     []string{destWasm, destWitDir},
		SourcePaths: []string{srcWitDir},
		Run: func() error {
			return sh.RunV(
				"golem-cli", "stubgen", "build",
				"--source-wit-root", srcWitDir,
				"--dest-wasm", destWasm,
				"--dest-wit-root", destWitDir,
			)
		},
	})
}

// AddStubDependency adds generated and built stub dependency to componentGolemCliAddStubDependency
func AddStubDependency(componentName, depComponentName string) error {
	stubTargetDir := filepath.Join(targetDir, "stub", depComponentName)
	srcWitDir := filepath.Join(stubTargetDir, "wit")
	dstComponentDir := filepath.Join(componentsDir, componentName)
	dstWitDir := filepath.Join(dstComponentDir, "wit")
	dstWitDepDir := filepath.Join(dstComponentDir, dstWitDir, "deps", fmt.Sprintf("%s_%s", pkgNs, componentName))
	dstWitDepStubDir := filepath.Join(dstComponentDir, dstWitDir, "deps", fmt.Sprintf("%s_%s-stub", pkgNs, componentName))

	return opRun(op{
		RunMessage:  fmt.Sprintf("Adding stub dependecy for %s to %s", depComponentName, componentName),
		SkipMessage: "add stub dependency",
		Targets:     []string{dstWitDepDir, dstWitDepStubDir},
		SourcePaths: []string{srcWitDir},
		Run: func() error {
			return sh.RunV(
				"golem-cli", "stubgen", "add-stub-dependency",
				"--overwrite",
				"--stub-wit-root", srcWitDir,
				"--dest-wit-root", dstWitDir,
			)
		},
	})
}

// StubCompose composes dependencies
func StubCompose(componentName, componentWasm, targetWasm string) error {
	buildTargetDir := filepath.Dir(componentWasm)
	dependencies := componentDeps[componentName]

	stubWasms := make([]string, len(dependencies))
	for i, componentName := range dependencies {
		stubTargetDir := filepath.Join(targetDir, "stub", componentName)
		stubWasms[i] = filepath.Join(stubTargetDir, "stub.wasm")
	}

	return opRun(op{
		RunMessage:  fmt.Sprintf("Composing %s into %s", fmt.Sprintf("[%s]", strings.Join(stubWasms, ", ")), componentName),
		SkipMessage: "composing",
		Targets:     []string{targetWasm},
		SourcePaths: append(stubWasms, componentWasm),
		Run: func() error {
			composeWasm := componentWasm
			if len(stubWasms) > 0 {
				srcWasm := componentWasm
				for i, stubWasm := range stubWasms {
					prevComposeWasm := composeWasm
					composeWasm = filepath.Join(
						buildTargetDir,
						fmt.Sprintf("compose-%d-%s.wasm", i+1, filepath.Base(dependencies[i])),
					)

					outBuf := &bytes.Buffer{}
					errBuff := &bytes.Buffer{}

					_, err := sh.Exec(
						nil, outBuf, errBuff,
						"golem-cli", "stubgen", "compose",
						"--source-wasm", srcWasm,
						"--stub-wasm", stubWasm,
						"--dest-wasm", composeWasm,
					)
					if err != nil {
						errString := errBuff.String()
						if strings.Contains(errString, "Error: no dependencies of component") &&
							strings.Contains(errString, "were found") {
							fmt.Printf("Skipping composing %s, not used\n", stubWasm)
							composeWasm = prevComposeWasm
							continue
						}

						fmt.Print(outBuf)
						fmt.Print(errBuff)

						return fmt.Errorf("StubCompose failed: %w", err)
					}
					srcWasm = composeWasm
				}
			}

			return copyFile(composeWasm, targetWasm)
		},
	})
}

// BuildComponent builds component by name
func BuildComponent(componentName string) error {
	componentDir := filepath.Join(componentsDir, componentName)
	witDir := filepath.Join(componentDir, "wit")
	bindingDir := filepath.Join(componentDir, "binding")
	buildTargetDir := filepath.Join(targetDir, "build", componentName)
	componentsTargetDir := filepath.Join(targetDir, "components")
	moduleWasm := filepath.Join(buildTargetDir, "module.wasm")
	embedWasm := filepath.Join(buildTargetDir, "embed.wasm")
	componentWasm := filepath.Join(buildTargetDir, "component.wasm")
	composedComponentWasm := filepath.Join(componentsTargetDir, fmt.Sprintf("%s.wasm", componentName))

	return serialRun(
		func() error { return os.MkdirAll(buildTargetDir, 0755) },
		func() error { return os.MkdirAll(componentsTargetDir, 0755) },
		func() error { return GenerateBinding(witDir, bindingDir) },
		func() error { return TinyGoBuildComponentBinary(componentDir, moduleWasm) },
		func() error { return WASMToolsComponentEmbed(witDir, moduleWasm, embedWasm) },
		func() error { return WASMToolsComponentNew(embedWasm, componentWasm) },
		func() error {
			return StubCompose(componentName, componentWasm, composedComponentWasm)
		},
	)
}

// GenerateBinding generates go binding from WIT
func GenerateBinding(witDir, bindingDir string) error {
	return opRun(op{
		RunMessage:  fmt.Sprintf("Generating bindings from %s into %s", witDir, bindingDir),
		SkipMessage: "binding generation",
		Targets:     []string{bindingDir},
		SourcePaths: []string{witDir},
		Run: func() error {
			return sh.RunV("wit-bindgen", "tiny-go", "--rename-package", "binding", "--out-dir", bindingDir, witDir)
		},
	})
}

// TinyGoBuildComponentBinary build wasm component binary with tiny go
func TinyGoBuildComponentBinary(componentDir, moduleWasm string) error {
	return opRun(op{
		RunMessage:  fmt.Sprintf("Building component binary with tiny go: %s", moduleWasm),
		SkipMessage: "tinygo component binary build",
		Targets:     []string{moduleWasm},
		SourcePaths: []string{componentsDir, libDir},
		Run: func() error {
			return sh.RunV(
				"tinygo", "build", "-target=wasi", "-tags=purego",
				"-o", moduleWasm,
				filepath.Join(componentDir, "main.go"),
			)
		},
	})
}

// WASMToolsComponentEmbed embeds type info into wasm component with wasm-tools
func WASMToolsComponentEmbed(witDir, moduleWasm, embedWasm string) error {
	return opRun(op{
		RunMessage:  fmt.Sprintf("Embedding component type info (%s, %s) -> %s", moduleWasm, witDir, embedWasm),
		SkipMessage: "wasm-tools component embed",
		Targets:     []string{embedWasm},
		SourcePaths: []string{witDir, moduleWasm},
		Run: func() error {
			return sh.RunV(
				"wasm-tools", "component", "embed",
				witDir, moduleWasm,
				"--output", embedWasm,
			)
		},
	})
}

// WASMToolsComponentNew create golem component with wasm-tools
func WASMToolsComponentNew(embedWasm, componentWasm string) error {
	return opRun(op{
		RunMessage:  fmt.Sprintf("Creating new component: %s", embedWasm),
		SkipMessage: "wasm-tools component new",
		Targets:     []string{componentWasm},
		SourcePaths: []string{embedWasm},
		Run: func() error {
			return sh.RunV(
				"wasm-tools", "component", "new",
				embedWasm,
				"-o", componentWasm,
				"--adapt", wasiSnapshotPreview1Adapter,
			)
		},
	})
}

// GenerateNewComponent generates a new component based on the component-template
func GenerateNewComponent(componentName string) error {
	err := sh.RunV("go", "run", "component-generator/main.go", pkgNs, componentName)
	if err != nil {
		return fmt.Errorf("generate new component failed for %s, %w", componentName, err)
	}

	return nil
}

// Clean cleans the projects
func Clean() error {
	fmt.Println("Cleaning...")

	paths := []string{targetDir}
	for _, componentName := range componentNames() {
		paths = append(paths, filepath.Join(componentsDir, componentName, "binding"))
	}

	for _, path := range paths {
		fmt.Printf("Deleting %s\n", path)
		err := os.RemoveAll(path)
		if err != nil {
			return fmt.Errorf("clean: remove all failed for %s, %w", path, err)
		}
	}

	return nil
}

// Deploy adds or updates all the components with golem-cli's default profile
func Deploy() error {
	componentsTargetDir := filepath.Join(targetDir, "components")
	for _, componentName := range componentNames() {
		wasm := filepath.Join(componentsTargetDir, fmt.Sprintf("%s.wasm", componentName))
		err := sh.RunV(
			"golem-cli", "component", "add",
			"--non-interactive",
			"--component-name", componentName,
			wasm,
		)
		if err != nil {
			return fmt.Errorf("deploy: failed for %s, %w", componentName, err)
		}
	}
	return nil
}

// TestIntegration tests the deployed components
func TestIntegration() error {
	err := sh.RunV("go", "test", "./integration", "-v")
	if err != nil {
		return fmt.Errorf("test integration failed: %w", err)
	}

	return nil
}

// componentNames returns component names based on directories found in the components directory
func componentNames() []string {
	var componentNames []string
	dirs, err := os.ReadDir(componentsDir)
	if err != nil {
		return nil
	}
	for _, dir := range dirs {
		componentNames = append(componentNames, dir.Name())
	}
	return componentNames
}

// stubComponentNames returns component names that need stubs based on the dependencies defined in componentDeps
func stubComponentNames() []string {
	componentNamesSet := make(map[string]struct{})
	for _, deps := range componentDeps {
		for _, dep := range deps {
			componentNamesSet[dep] = struct{}{}
		}
	}

	var componentNames []string
	for comp := range componentNamesSet {
		componentNames = append(componentNames, comp)
	}
	sort.Strings(componentNames)
	return componentNames
}

func copyFile(srcFileName, dstFileName string) error {
	src, err := os.Open(srcFileName)
	if err != nil {
		return fmt.Errorf("copyFile: open failed for %s, %w", srcFileName, err)
	}
	defer func() { _ = src.Close() }()

	dst, err := os.Create(dstFileName)
	if err != nil {
		return fmt.Errorf("copyFile: create failed for %s, %w", srcFileName, err)
	}
	defer func() { _ = dst.Close() }()

	_, err = io.Copy(dst, src)
	if err != nil {
		return fmt.Errorf("copyFile: copy failed from %s to %s, %w", srcFileName, dstFileName, err)
	}

	return nil
}

func serialRun(fs ...func() error) error {
	for i, f := range fs {
		err := f()
		if err != nil {
			return fmt.Errorf("serialRun: step %d failed: %w", i+1, err)
		}
	}
	return nil
}

type op struct {
	RunMessage  string
	SkipMessage string
	Targets     []string
	SourcePaths []string
	Run         func() error
}

func opRun(op op) error {
	var run bool
	if len(op.Targets) == 0 {
		run = true
	} else {
		run = false
		for _, t := range op.Targets {
			var err error
			run, err = target.Dir(t, op.SourcePaths...)
			if err != nil {
				return err
			}
			if run {
				break
			}
		}
	}

	if !run {
		var targets string
		if len(op.Targets) == 1 {
			targets = op.Targets[0]
		} else {
			targets = fmt.Sprintf("(%s)", strings.Join(op.Targets, ", "))
		}
		fmt.Printf("%s is up to date, skipping %s\n", targets, op.SkipMessage)
		return nil
	}

	fmt.Println(op.RunMessage)
	return op.Run()
}
