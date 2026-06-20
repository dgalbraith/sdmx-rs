<details>
<summary>XSD contract: <code>DataflowReferenceType</code> (SDMX 3.0 and 3.1)</summary>

```xml
	<xs:simpleType name="DataflowReferenceType">
		<xs:annotation>
			<xs:documentation>A reference type for a dataflow.</xs:documentation>
		</xs:annotation>
		<xs:restriction base="StructureUsageReferenceType">
			<xs:pattern value=".+\.datastructure\.Dataflow=.+"/>
		</xs:restriction>
	</xs:simpleType>
```

</details>
