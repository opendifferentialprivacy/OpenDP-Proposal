# OpenDP Release Schemas

The end product of a differentially private analysis is a "release" which the OpenDP framework returns in JSON format.

For both validation and tracking of the OpenDP release structure, this directory includes versioned [JSON Schemas](https://json-schema.org/).

Directory structure:

```
|-- HISTORY.md
|-- test_schemas.py 
|-- version-0.2.3
|   |-- ddi_conversion_0_2_3
|       |-- ddi_covert.py 
|       |-- (other files)
|   |-- release-schema.json
|   |-- release-example-files
|       |-- example_01.json
|       |-- example_02.json
|       |-- example_nn.json 
|-- version-0.2.0
|   |-- ddi_conversion_0_2_0
|       |-- ddi_covert.py 
|       |-- (other files)
    |-- release-schema.json
    |-- release-example-files
        |-- example_01.json
        |-- example_02.json 
        |-- example_nn.json 

```

## Notes
  - `release-schema.json` 
    - A JSON schema that can validate each of the files in the related `release-example-files` directory
  - `test_schemas.py` 
    - Test each example file against its schema to check validity
    - Run DDI conversions, converting the example releases to DDI format
  - `ddi_convert_0_2_0.py`
    - Given an example release file for input, convert it into a [DDI in XML format](https://en.wikipedia.org/wiki/Data_Documentation_Initiative)
