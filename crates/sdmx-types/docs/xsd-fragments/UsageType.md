<details>
<summary>XSD contract: <code>UsageType</code> (SDMX 3.0 and 3.1)</summary>

```xml
	<xs:simpleType name="UsageType">
		<xs:annotation>
			<xs:documentation>An enumeration of optional | mandatory to indicate the usage of an attribute or measure.</xs:documentation>
		</xs:annotation>
		<xs:restriction base="xs:string">
			<xs:enumeration value="mandatory"/>
			<xs:enumeration value="optional"/>
		</xs:restriction>
	</xs:simpleType>
```

</details>
