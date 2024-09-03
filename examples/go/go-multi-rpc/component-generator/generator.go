package main

import (
	"fmt"
	"io"
	"io/fs"
	"os"
	"path/filepath"
	"strings"
)

func generate() {
	if len(os.Args) != 3 {
		exit(0, fmt.Sprintf("Usage: %s <package-namespace> <component-name>", os.Args[0]))
	}

	componentTemplateRoot := "component-template/component"
	pkgNs := os.Args[1]
	componentName := os.Args[2]
	componentDir := filepath.Join("components", componentName)

	_, err := os.Stat(componentDir)
	if err == nil {
		exit(1, fmt.Sprintf("Target component directory already exists: %s", componentDir))
	}
	if err != nil && !os.IsNotExist(err) {
		exit(1, err.Error())
	}

	err = os.MkdirAll(componentDir, 0755)
	if err != nil {
		exit(1, err.Error())
	}

	err = fs.WalkDir(
		os.DirFS(componentTemplateRoot),
		".",
		func(path string, d fs.DirEntry, err error) error {
			srcFilePath := filepath.Join(componentTemplateRoot, path)
			fileInfo, err := os.Stat(srcFilePath)
			if err != nil {
				return fmt.Errorf("stat failed for template %s, %w", srcFilePath, err)
			}

			if fileInfo.IsDir() {
				return nil
			}

			switch path {
			case "main.go":
				err = generateFile(pkgNs, componentName, srcFilePath, filepath.Join(componentDir, path))
			case "wit/component.wit":
				err = generateFile(pkgNs, componentName, srcFilePath, filepath.Join(componentDir, "wit", componentName+".wit"))
			default:
				err = copyFile(srcFilePath, filepath.Join(componentDir, path))
			}
			if err != nil {
				return fmt.Errorf("template generation failed for %s, %w", srcFilePath, err)
			}

			return nil
		})
	if err != nil {
		exit(1, err.Error())
	}
}

func generateFile(pkgOrg, componentName, srcFileName, dstFileName string) error {
	pascalPkgOrg := dashToPascal(pkgOrg)
	pascalComponentName := dashToPascal(componentName)

	fmt.Printf("Generating from %s to %s\n", srcFileName, dstFileName)

	contentsBs, err := os.ReadFile(srcFileName)
	if err != nil {
		return fmt.Errorf("generateFile: read file failed for %s, %w", srcFileName, err)
	}

	contents := string(contentsBs)

	contents = strings.ReplaceAll(contents, "comp-name", componentName)
	contents = strings.ReplaceAll(contents, "pck-ns", pkgOrg)
	contents = strings.ReplaceAll(contents, "CompName", pascalComponentName)
	contents = strings.ReplaceAll(contents, "PckNs", pascalPkgOrg)

	dstDir := filepath.Dir(dstFileName)
	err = os.MkdirAll(dstDir, 0755)
	if err != nil {
		return fmt.Errorf("generateFile: mkdir failed for %s, %w", dstDir, err)
	}

	err = os.WriteFile(dstFileName, []byte(contents), 0644)
	if err != nil {
		return fmt.Errorf("generateFile: write file failed for %s, %w", dstFileName, err)
	}

	return nil
}

func dashToPascal(s string) string {
	parts := strings.Split(s, "-")
	for i, part := range parts {
		if len(part) > 0 {
			parts[i] = strings.ToUpper(string(part[0])) + part[1:]
		}
	}
	return strings.Join(parts, "")
}

func copyFile(srcFileName, dstFileName string) error {
	fmt.Printf("Copy %s to %s\n", srcFileName, dstFileName)

	src, err := os.Open(srcFileName)
	if err != nil {
		return fmt.Errorf("copyFile: open failed for %s, %w", srcFileName, err)
	}
	defer func() { _ = src.Close() }()

	dstDir := filepath.Dir(dstFileName)
	err = os.MkdirAll(dstDir, 0755)
	if err != nil {
		return fmt.Errorf("copyFile: mkdir failed for %s, %w", dstDir, err)
	}

	dst, err := os.Create(dstFileName)
	if err != nil {
		return fmt.Errorf("copyFile: create file failed for %s, %w", dstFileName, err)
	}
	defer func() { _ = dst.Close() }()

	_, err = io.Copy(dst, src)
	if err != nil {
		return fmt.Errorf("copyFile: copy failed from %s to %s, %w", srcFileName, dstFileName, err)
	}

	return nil
}

func exit(code int, message string) {
	fmt.Println(message)
	os.Exit(code)
}
