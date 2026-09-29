// scandir_filter.go - Example demonstrating filter options with Scandir.
package main

import (
	"fmt"
	"os"

	cscandir "github.com/brmmm3/scandir-rs/goscandir"
)

func main() {
	root := "."
	if len(os.Args) > 1 {
		root = os.Args[1]
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
	fmt.Println("  Excluded dirs: vendor, build, dist, .git, node_modules, target\n")

	fmt.Printf("Found %d entries\n", len(result.Entries))
	fmt.Printf("Directories: %d  Files: %d\n", result.Statistics.Dirs, result.Statistics.Files)

	fileCount := 0
	for _, entry := range result.Entries {
		if entry.IsFile {
			fmt.Printf("  %s\n", entry.Path)
			fileCount++
		}
	}
	fmt.Printf("Total matching files: %d\n", fileCount)
}