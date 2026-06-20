<details>
<summary>XSD contract: <code>CubeRegionKeyType</code> (SDMX 3.0 and 3.1)</summary>

```xml
	<xs:complexType name="CubeRegionKeyType">
		<xs:annotation>
			<xs:documentation>CubeRegionKeyType is a type for providing a set of values for a dimension for the purpose of defining a data cube region. A set of distinct value can be provided, or if this dimension is represented as time, and time range can be specified.</xs:documentation>
		</xs:annotation>
		<xs:complexContent>
			<xs:restriction base="MemberSelectionType">
				<xs:choice>
					<xs:element name="Value" type="CubeKeyValueType" maxOccurs="unbounded"/>
					<xs:element name="TimeRange" type="TimeRangeValueType"/>
				</xs:choice>
				<xs:attribute name="id" type="common:SingleNCNameIDType" use="required"/>
			</xs:restriction>
		</xs:complexContent>
	</xs:complexType>
```

</details>
