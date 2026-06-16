<details>
<summary>XSD contract: <code>AnyCodelistReferenceType</code> (SDMX 3.0 and 3.1)</summary>

```xml
	<xs:simpleType name="AnyCodelistReferenceType">
		<xs:annotation>
			<xs:documentation>A reference type for a codelist or value list.</xs:documentation>
		</xs:annotation>
		<xs:restriction base="MaintainableUrnReferenceType">
			<xs:pattern value=".+\.codelist\.Codelist=.+"/>
			<xs:pattern value=".+\.codelist\.ValueList=.+"/>
		</xs:restriction>
	</xs:simpleType>
```

</details>
