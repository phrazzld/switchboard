#!/usr/bin/env python3
"""Pre-commit hook to check file length."""

import sys
import argparse
import os
import re
from typing import List, Optional, Sequence


def main(argv: Optional[Sequence[str]] = None) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument('--warn-lines', type=int, default=500,
                        help='Warning threshold in lines (default: 500)')
    parser.add_argument('--max-lines', type=int, default=1000,
                        help='Error threshold in lines (default: 1000)')
    parser.add_argument('--skip', action='append', default=[],
                        help='Files to skip (can include glob patterns)')
    parser.add_argument('filenames', nargs='*', help='Filenames to check')
    args = parser.parse_args(argv)

    retval = 0
    has_warnings = False
    warnings = []
    errors = []
    
    # Convert skip patterns to regex patterns
    skip_patterns = []
    for pattern in args.skip:
        # Convert glob pattern to regex pattern
        regex_pattern = pattern.replace('.', '\\.')
        regex_pattern = regex_pattern.replace('*', '.*')
        regex_pattern = regex_pattern.replace('?', '.')
        skip_patterns.append(re.compile(regex_pattern))

    for filename in args.filenames:
        # Get relative path for cleaner output
        rel_filename = os.path.relpath(filename)
        
        # Check if file should be skipped
        skip_file = False
        for pattern in skip_patterns:
            if pattern.search(rel_filename):
                skip_file = True
                break
        
        if skip_file:
            continue
            
        try:
            with open(filename, 'rb') as f:
                line_count = sum(1 for _ in f)
            
            if line_count >= args.max_lines:
                msg = f'ERROR: {rel_filename} has {line_count} lines, which exceeds the maximum of {args.max_lines}'
                print(msg)
                errors.append(msg)
                retval = 1
            elif line_count >= args.warn_lines:
                msg = f'WARNING: {rel_filename} has {line_count} lines, which exceeds the warning threshold of {args.warn_lines}'
                print(msg)
                warnings.append(msg)
                has_warnings = True
        except (IOError, OSError) as e:
            msg = f'Failed to check {rel_filename}: {e}'
            print(msg)
            errors.append(msg)
            retval = 1
    
    # Summary
    if warnings:
        print("\nWarnings summary:")
        for warning in warnings:
            print(f"- {warning}")
    
    if errors:
        print("\nErrors summary:")
        for error in errors:
            print(f"- {error}")
    
    return retval


if __name__ == '__main__':
    sys.exit(main())