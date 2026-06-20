<details>
<summary>XSD contract: <code>DataKeySetType</code> (SDMX 3.0 and 3.1)</summary>

```xml
<xs:complexType name="DataKeySetType">
		<xs:annotation>
			<xs:documentation>DataKeySetType defines a collection of full or partial data keys (dimension values).</xs:documentation>
		</xs:annotation>
		<xs:sequence>
			<xs:element name="Key" type="DataKeyType" maxOccurs="unbounded">
				<xs:annotation>
					<xs:documentation>Key contains a set of dimension values which identify a full set of data.</xs:documentation>
				</xs:annotation>
			</xs:element>
		</xs:sequence>
		<xs:attribute name="isIncluded" type="xs:boolean" use="required">
			<xs:annotation>
				<xs:documentation>The isIncluded attribute indicates whether the keys defined in this key set are inclusive or exclusive to the constraint.</xs:documentation>
			</xs:annotation>
		</xs:attribute>
	</xs:complexType>
```

</details>
