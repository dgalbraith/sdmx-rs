<details>
<summary>XSD contract: <code>CodelistReferenceType</code> (SDMX 3.0 and 3.1)</summary>

```xml
	<xs:simpleType name="CodelistReferenceType">
		<xs:annotation>
			<xs:documentation>A reference type for a codelist.</xs:documentation>
		</xs:annotation>
		<xs:restriction base="AnyCodelistReferenceType">
			<xs:pattern value=".+\.codelist\.Codelist=.+"/>
		</xs:restriction>
	</xs:simpleType>
```

</details>
