<details>
<summary>XSD contract: <code>ValueListReferenceType</code> (SDMX 3.0 and 3.1)</summary>

```xml
	<xs:simpleType name="ValueListReferenceType">
		<xs:annotation>
			<xs:documentation>A reference type for a value list.</xs:documentation>
		</xs:annotation>
		<xs:restriction base="AnyCodelistReferenceType">
			<xs:pattern value=".+\.codelist\.ValueList=.+"/>
		</xs:restriction>
	</xs:simpleType>
```

</details>
