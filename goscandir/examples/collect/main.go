// Example demonstrating the Collect function.
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
	fmt.Printf("Metadata: %s\n", map[bool]string{true: "extended", false: "base"}[options.ReturnType == cscandir.ReturnTypeExt])
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
