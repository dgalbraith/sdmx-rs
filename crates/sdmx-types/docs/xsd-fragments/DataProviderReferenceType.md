<details>
<summary>XSD contract: <code>DataProviderReferenceType</code> (SDMX 3.0 and 3.1)</summary>

```xml
	<xs:simpleType name="DataProviderReferenceType">
		<xs:annotation>
			<xs:documentation>A reference type for a data provider.</xs:documentation>
		</xs:annotation>
		<xs:restriction base="OrganisationReferenceType">
			<xs:pattern value=".+\.base\.DataProvider=.+:DATA_PROVIDERS\(.+\).+"/>
		</xs:restriction>
	</xs:simpleType>
```

</details>
