<details>
<summary>XSD contract: <code>CubeKeyValueType</code> (SDMX 3.0 and 3.1)</summary>

```xml
	<xs:complexType name="CubeKeyValueType">
		<xs:annotation>
			<xs:documentation>CubeKeyValueType derives from the SimpleValueType, but does not allow for a locale (xml:lang).</xs:documentation>
		</xs:annotation>
		<xs:simpleContent>
			<xs:restriction base="SimpleComponentValueType">
				<xs:attribute ref="xml:lang" use="prohibited"/>
			</xs:restriction>
		</xs:simpleContent>
	</xs:complexType>
```

</details>
