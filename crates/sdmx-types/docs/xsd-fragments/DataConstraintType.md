<details>
<summary>XSD contract: <code>DataConstraintType</code> (SDMX 3.0 and 3.1)</summary>

```xml
	<xs:complexType name="DataConstraintType">
		<xs:annotation>
			<xs:documentation>DataConstraintType defines the structure of a data constraint. A data constraint contains the allowed values for the referencing artefact. These values are expressed either as sets of keys (DataKeySets) or sets of component values (CubeRegion) constructed from a data structure definition. Data constraints can be used, e.g., for validation or for defining a partial code list.</xs:documentation>
		</xs:annotation>
		<xs:complexContent>
			<xs:extension base="DataConstraintBaseType">
				<xs:sequence>
					<xs:element name="DataKeySet" type="DataKeySetType" minOccurs="0" maxOccurs="unbounded">
						<xs:annotation>
							<xs:documentation>DataKeySet defines a full, distinct set of dimension values and the attribute values associated with the key.</xs:documentation>
						</xs:annotation>
					</xs:element>
					<xs:element name="CubeRegion" type="CubeRegionType" minOccurs="0" maxOccurs="2">
						<xs:annotation>
							<xs:documentation>CubeRegion defines a slice of the data set (dimensions and attribute values) for the constrained artefact. A set of included or excluded regions can be described.</xs:documentation>
						</xs:annotation>
					</xs:element>
				</xs:sequence>
			</xs:extension>
		</xs:complexContent>
	</xs:complexType>
```

</details>
