"""
Test all schemas and their related example files for validity.

ref: https://github.com/Julian/jsonschema
"""

import json

from jsonschema import validate


def validate_report(instance):

    with open('version-0.1.0-draft/release-schema.json', 'r') as infile:
        schema = json.load(infile)

    return validate(instance=instance, schema=schema)


if __name__ == '__main__':

    with open('version-0.1.0-draft/release-example-files/example_01.json', 'r') as infile:
        report = json.load(infile)

    validate_report(report)
    print("Validation succeeded")
