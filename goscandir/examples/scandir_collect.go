// scandir_collect.go - Example demonstrating the Collect function.
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

	options := &cscandir.Options{
		Sorted:     true,
		SkipHidden: true,
		ReturnType: cscandir.ReturnTypeExt,
	}

	result, err := cscandir.Collect(root, options)
	if err != nil {
		fmt.Fprintf(os.Stderr, "error: %v\n", err)
		os.Exit(1)
	}

	fmt.Printf("Found %d entries\n", len(result.Entries))
	fmt.Printf("Directories: %d, Files: %d, Symlinks: %d\n",
		result.Statistics.Dirs, result.Statistics.Files, result.Statistics.Slinks)
	fmt.Printf("Duration: %.4fs\n\n", result.Statistics.Duration)

	for i, entry := range result.Entries {
		if i >= 10 {
			fmt.Printf("... and %d more entries\n", len(result.Entries)-10)
			break
		}
		kind := "?"
		switch {
		case entry.IsDir:
			kind = "D"
		case entry.IsFile:
			kind = "F"
		case entry.IsSymlink:
			kind = "L"
		}
		fmt.Printf("  %s %s\n", kind, entry.Path)
		if entry.HasExt {
			fmt.Printf("    inode=%d mode=%o size=%d\n", entry.Ino, entry.Mode, entry.Size)
		}
	}

	if len(result.Errors) > 0 {
		fmt.Println("\nScan errors:")
		for _, err := range result.Errors {
			fmt.Printf("  %s\n", err)
		}
	}
}