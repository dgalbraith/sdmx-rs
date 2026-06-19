<details>
<summary>XSD contract: <code>ConceptReferenceType</code> (SDMX 3.0 and 3.1)</summary>

```xml
	<xs:simpleType name="ConceptReferenceType">
		<xs:annotation>
			<xs:documentation>A reference type for a concept.</xs:documentation>
		</xs:annotation>
		<xs:restriction base="ComponentUrnReferenceType">
			<xs:pattern value=".+\.conceptscheme\.Concept=.+"/>
		</xs:restriction>
	</xs:simpleType>
```

</details>
