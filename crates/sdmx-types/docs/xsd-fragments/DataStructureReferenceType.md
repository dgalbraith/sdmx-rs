<details>
<summary>XSD contract: <code>DataStructureReferenceType</code> (SDMX 3.0 and 3.1)</summary>

```xml
	<xs:simpleType name="DataStructureReferenceType">
		<xs:annotation>
			<xs:documentation>A reference type for a data structure.</xs:documentation>
		</xs:annotation>
		<xs:restriction base="StructureReferenceType">
			<xs:pattern value=".+\.datastructure\.DataStructure=.+"/>
		</xs:restriction>
	</xs:simpleType>
```

</details>
