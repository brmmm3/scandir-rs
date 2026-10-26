// Example demonstrating filter options with Collect.
package main

import (
	"fmt"
	"os"

	cscandir "github.com/brmmm3/scandir-rs/goscandir"
)

func main() {
	root, ok := parseRoot()
	if !ok {
		fmt.Fprintf(os.Stderr, "Usage: %s [ROOT]\n", os.Args[0])
		os.Exit(2)
	}

	// Filter for only .go and .mod files, exclude build directories
	options := &cscandir.Options{
		FileInclude: []string{"*.go", "go.mod", "go.sum"},
		DirExclude:  []string{"vendor", "build", "dist", ".git", "node_modules", "target"},
		Sorted:      true,
		ReturnType:  cscandir.ReturnTypeExt,
	}

	result, err := cscandir.Collect(root, options)
	if err != nil {
		fmt.Fprintf(os.Stderr, "error: %v\n", err)
		os.Exit(1)
	}

	fmt.Printf("Filtering: %s\n", root)
	fmt.Println("  File patterns: *.go, go.mod, go.sum")
	fmt.Println("  Excluded dirs: vendor, build, dist, .git, node_modules, target")
	fmt.Println()

	fmt.Printf("Found %d entries\n", len(result.Entries))
	files, dirs := 0, 0
	fileCount := 0
	for i, entry := range result.Entries {
		if entry.IsFile {
			fmt.Printf("  %s\n", entry.Path)
			fileCount++
			files++
		} else if entry.IsDir && i != 0 {
			dirs++
		}
	}
	fmt.Printf("Directories: %d  Files: %d\n", dirs, files)
	fmt.Printf("Total matching files: %d\n", fileCount)
	if len(result.Errors) != 0 {
		for _, scanErr := range result.Errors {
			fmt.Fprintf(os.Stderr, "scan error: %s\n", scanErr)
		}
		os.Exit(1)
	}
}

func parseRoot() (string, bool) {
	if len(os.Args) > 2 {
		return "", false
	}
	if len(os.Args) == 2 {
		return os.Args[1], true
	}
	return ".", true
}
